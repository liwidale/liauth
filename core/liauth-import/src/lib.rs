mod aegis;
mod authy;
mod google;
mod twofas;
mod uri_list;

pub use aegis::import_aegis;
pub use authy::import_authy;
pub use google::import_google_migration;
pub use twofas::import_twofas;
pub use uri_list::import_uri_list;

use liauth_core::Account;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ImportError {
    #[error("unrecognized format")]
    Unrecognized,
    #[error("password required")]
    PasswordRequired,
    #[error("wrong password")]
    WrongPassword,
    #[error("malformed data: {0}")]
    Malformed(String),
    #[error("nothing to import")]
    Empty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportSource {
    LiAuthBackup,
    GoogleAuthenticator,
    Aegis,
    TwoFas,
    Authy,
    UriList,
}

pub struct ImportResult {
    pub source: ImportSource,
    pub accounts: Vec<Account>,
}

pub fn detect_and_import(data: &[u8], password: Option<&str>) -> Result<ImportResult, ImportError> {
    if let Ok(text) = std::str::from_utf8(data) {
        let trimmed = text.trim();
        if trimmed.starts_with("otpauth-migration://") {
            return Ok(ImportResult {
                source: ImportSource::GoogleAuthenticator,
                accounts: import_google_migration(trimmed)?,
            });
        }
        if trimmed.starts_with("otpauth://") {
            return Ok(ImportResult {
                source: ImportSource::UriList,
                accounts: import_uri_list(trimmed)?,
            });
        }
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(trimmed) {
            if json.get("services").is_some() || json.get("servicesEncrypted").is_some() {
                return Ok(ImportResult {
                    source: ImportSource::TwoFas,
                    accounts: import_twofas(trimmed, password)?,
                });
            }
            if json.get("header").is_some() && json.get("db").is_some() {
                return Ok(ImportResult {
                    source: ImportSource::Aegis,
                    accounts: import_aegis(trimmed, password)?,
                });
            }
            if json.is_array() {
                return Ok(ImportResult {
                    source: ImportSource::Authy,
                    accounts: import_authy(trimmed)?,
                });
            }
        }
    }
    Err(ImportError::Unrecognized)
}
