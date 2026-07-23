//! Minimal WebDAV client for pushing encrypted backups to Nextcloud, a NAS
//! or any other WebDAV endpoint. Only what the app needs: PUT, GET, MKCOL
//! and an OPTIONS-based connectivity check, all over HTTPS (or HTTP for
//! LAN-only servers) with Basic auth.
//!
//! The uploaded bytes are always a sealed LiAuth backup envelope, so the
//! server only ever sees ciphertext.

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;

use crate::SyncError;

const TIMEOUT_SECONDS: u64 = 20;

#[derive(Debug, Clone)]
pub struct WebDavConfig {
    /// Full URL of the target directory, e.g.
    /// `https://cloud.example.com/remote.php/dav/files/user/liauth`.
    pub url: String,
    pub username: String,
    pub password: String,
}

impl WebDavConfig {
    fn auth_header(&self) -> String {
        let credentials = BASE64.encode(format!("{}:{}", self.username, self.password));
        format!("Basic {credentials}")
    }

    fn file_url(&self, file_name: &str) -> String {
        let base = self.url.trim_end_matches('/');
        format!("{base}/{file_name}")
    }
}

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(TIMEOUT_SECONDS))
        .build()
}

/// Verifies that the server answers and the credentials are accepted.
pub fn check_connection(config: &WebDavConfig) -> Result<(), SyncError> {
    let response = agent()
        .request("OPTIONS", &config.url)
        .set("Authorization", &config.auth_header())
        .call()
        .map_err(request_error)?;
    let _ = response;
    Ok(())
}

/// Creates the remote directory when it does not exist yet. An existing
/// directory (405) is not an error.
pub fn ensure_directory(config: &WebDavConfig) -> Result<(), SyncError> {
    match agent()
        .request("MKCOL", &config.url)
        .set("Authorization", &config.auth_header())
        .call()
    {
        Ok(_) => Ok(()),
        Err(ureq::Error::Status(405, _)) => Ok(()),
        Err(e) => Err(request_error(e)),
    }
}

/// Uploads an encrypted backup, replacing the previous one.
pub fn upload(config: &WebDavConfig, file_name: &str, bytes: &[u8]) -> Result<(), SyncError> {
    ensure_directory(config)?;
    agent()
        .put(&config.file_url(file_name))
        .set("Authorization", &config.auth_header())
        .set("Content-Type", "application/octet-stream")
        .send_bytes(bytes)
        .map_err(request_error)?;
    Ok(())
}

/// Downloads a backup previously stored with [`upload`].
pub fn download(config: &WebDavConfig, file_name: &str) -> Result<Vec<u8>, SyncError> {
    let response = agent()
        .get(&config.file_url(file_name))
        .set("Authorization", &config.auth_header())
        .call()
        .map_err(request_error)?;
    let mut bytes = Vec::new();
    response
        .into_reader()
        .read_to_end(&mut bytes)
        .map_err(SyncError::Io)?;
    Ok(bytes)
}

fn request_error(error: ureq::Error) -> SyncError {
    match error {
        ureq::Error::Status(401 | 403, _) => SyncError::WebDav("authentication rejected".into()),
        ureq::Error::Status(code, _) => SyncError::WebDav(format!("server returned {code}")),
        ureq::Error::Transport(t) => SyncError::WebDav(t.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader, Read, Write};
    use std::net::TcpListener;

    fn config(port: u16, path: &str) -> WebDavConfig {
        WebDavConfig {
            url: format!("http://127.0.0.1:{port}{path}"),
            username: "user".into(),
            password: "secret".into(),
        }
    }

    /// One-shot HTTP server that records the requests it saw.
    fn serve(listener: TcpListener, responses: Vec<&'static str>) -> std::thread::JoinHandle<Vec<String>> {
        std::thread::spawn(move || {
            let mut seen = Vec::new();
            for response in responses {
                let (mut stream, _) = listener.accept().unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut request_line = String::new();
                reader.read_line(&mut request_line).unwrap();
                let mut content_length = 0usize;
                loop {
                    let mut line = String::new();
                    reader.read_line(&mut line).unwrap();
                    let lower = line.to_ascii_lowercase();
                    if let Some(value) = lower.strip_prefix("content-length:") {
                        content_length = value.trim().parse().unwrap_or(0);
                    }
                    if line == "\r\n" || line.is_empty() {
                        break;
                    }
                }
                let mut body = vec![0u8; content_length];
                if content_length > 0 {
                    reader.read_exact(&mut body).unwrap();
                }
                seen.push(request_line.trim().to_string());
                stream.write_all(response.as_bytes()).unwrap();
            }
            seen
        })
    }

    #[test]
    fn upload_issues_mkcol_then_put() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let handle = serve(
            listener,
            vec![
                "HTTP/1.1 201 Created\r\nContent-Length: 0\r\n\r\n",
                "HTTP/1.1 201 Created\r\nContent-Length: 0\r\n\r\n",
            ],
        );
        upload(&config(port, "/dav/liauth"), "backup.liauthbackup", b"cipher").unwrap();
        let seen = handle.join().unwrap();
        assert!(seen[0].starts_with("MKCOL /dav/liauth"));
        assert!(seen[1].starts_with("PUT /dav/liauth/backup.liauthbackup"));
    }

    #[test]
    fn wrong_credentials_reported() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let handle = serve(
            listener,
            vec!["HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\n\r\n"],
        );
        let error = check_connection(&config(port, "/dav")).unwrap_err();
        assert!(matches!(error, SyncError::WebDav(ref m) if m.contains("authentication")));
        handle.join().unwrap();
    }

    #[test]
    fn download_roundtrip() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let handle = serve(
            listener,
            vec!["HTTP/1.1 200 OK\r\nContent-Length: 6\r\n\r\ncipher"],
        );
        let bytes = download(&config(port, "/dav"), "backup.liauthbackup").unwrap();
        assert_eq!(bytes, b"cipher");
        handle.join().unwrap();
    }
}
