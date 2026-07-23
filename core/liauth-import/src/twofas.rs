use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use liauth_core::time::unix_now;
use liauth_core::{base32, Account, Algorithm, TokenKind};
use pbkdf2::pbkdf2_hmac;
use serde::Deserialize;
use sha2::Sha256;

use crate::ImportError;

const PBKDF2_ITERATIONS: u32 = 10000;

#[derive(Deserialize)]
struct TwoFasFile {
    #[serde(default)]
    services: Vec<TwoFasService>,
    #[serde(default, rename = "servicesEncrypted")]
    services_encrypted: Option<String>,
}

#[derive(Deserialize)]
struct TwoFasService {
    #[serde(default)]
    name: String,
    secret: String,
    #[serde(default)]
    otp: Option<TwoFasOtp>,
}

#[derive(Deserialize)]
struct TwoFasOtp {
    #[serde(default)]
    account: Option<String>,
    #[serde(default)]
    issuer: Option<String>,
    #[serde(default)]
    digits: Option<u32>,
    #[serde(default)]
    period: Option<u32>,
    #[serde(default)]
    algorithm: Option<String>,
    #[serde(default, rename = "tokenType")]
    token_type: Option<String>,
    #[serde(default)]
    counter: Option<u64>,
}

pub fn import_twofas(text: &str, password: Option<&str>) -> Result<Vec<Account>, ImportError> {
    let file: TwoFasFile = serde_json::from_str(text).map_err(|e| ImportError::Malformed(e.to_string()))?;

    let services = if !file.services.is_empty() {
        file.services
    } else if let Some(encrypted) = file.services_encrypted {
        let password = password.ok_or(ImportError::PasswordRequired)?;
        decrypt_services(&encrypted, password)?
    } else {
        return Err(ImportError::Empty);
    };

    let mut accounts = Vec::new();
    for service in services {
        let secret = base32::decode(&service.secret)
            .map_err(|_| ImportError::Malformed("invalid service secret".into()))?;
        let otp = service.otp;
        let issuer = otp
            .as_ref()
            .and_then(|o| o.issuer.clone())
            .filter(|i| !i.is_empty())
            .unwrap_or(service.name);
        let name = otp.as_ref().and_then(|o| o.account.clone()).unwrap_or_default();

        let mut account = Account::new(issuer, name, secret, unix_now());
        if let Some(otp) = otp {
            account.algorithm = otp
                .algorithm
                .as_deref()
                .and_then(Algorithm::parse)
                .unwrap_or_default();
            account.digits = otp.digits.filter(|d| (4..=10).contains(d)).unwrap_or(6);
            account.kind = match otp.token_type.as_deref().map(str::to_ascii_lowercase).as_deref() {
                Some("hotp") => TokenKind::Hotp {
                    counter: otp.counter.unwrap_or(0),
                },
                Some("steam") => TokenKind::Steam,
                _ => TokenKind::Totp {
                    period: otp.period.filter(|p| *p >= 5).unwrap_or(30),
                },
            };
        }
        accounts.push(account);
    }
    if accounts.is_empty() {
        return Err(ImportError::Empty);
    }
    Ok(accounts)
}

fn decrypt_services(encrypted: &str, password: &str) -> Result<Vec<TwoFasService>, ImportError> {
    let mut parts = encrypted.split(':');
    let ciphertext = decode_part(parts.next())?;
    let salt = decode_part(parts.next())?;
    let iv = decode_part(parts.next())?;

    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, PBKDF2_ITERATIONS, &mut key);

    let cipher = Aes256Gcm::new(&key.into());
    let plaintext = cipher
        .decrypt(
            Nonce::from_slice(&iv),
            Payload {
                msg: &ciphertext,
                aad: &[],
            },
        )
        .map_err(|_| ImportError::WrongPassword)?;
    serde_json::from_slice(&plaintext).map_err(|e| ImportError::Malformed(e.to_string()))
}

fn decode_part(part: Option<&str>) -> Result<Vec<u8>, ImportError> {
    let part = part.ok_or_else(|| ImportError::Malformed("missing encrypted segment".into()))?;
    B64.decode(part.as_bytes())
        .map_err(|_| ImportError::Malformed("invalid base64 segment".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imports_plain_backup() {
        let json = r#"{
            "services": [
                {
                    "name": "GitHub",
                    "secret": "MZXW6YTBOI",
                    "otp": {"account": "liwidale", "issuer": "GitHub", "digits": 6, "period": 30, "algorithm": "SHA1", "tokenType": "TOTP"}
                }
            ],
            "schemaVersion": 4
        }"#;
        let accounts = import_twofas(json, None).unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].issuer, "GitHub");
        assert_eq!(accounts[0].name, "liwidale");
    }

    #[test]
    fn encrypted_roundtrip() {
        let services = r#"[{"name":"GitHub","secret":"MZXW6YTBOI","otp":{"account":"me","issuer":"GitHub","tokenType":"TOTP"}}]"#;
        let salt = vec![7u8; 16];
        let iv = vec![9u8; 12];
        let mut key = [0u8; 32];
        pbkdf2_hmac::<Sha256>(b"backup pass", &salt, PBKDF2_ITERATIONS, &mut key);
        let cipher = Aes256Gcm::new(&key.into());
        let ciphertext = cipher
            .encrypt(
                Nonce::from_slice(&iv),
                Payload {
                    msg: services.as_bytes(),
                    aad: &[],
                },
            )
            .unwrap();
        let encoded = format!(
            "{}:{}:{}",
            B64.encode(&ciphertext),
            B64.encode(&salt),
            B64.encode(&iv)
        );
        let json = format!(r#"{{"servicesEncrypted": "{encoded}"}}"#);

        assert!(matches!(
            import_twofas(&json, None),
            Err(ImportError::PasswordRequired)
        ));
        assert!(matches!(
            import_twofas(&json, Some("wrong")),
            Err(ImportError::WrongPassword)
        ));
        let accounts = import_twofas(&json, Some("backup pass")).unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].name, "me");
    }
}
