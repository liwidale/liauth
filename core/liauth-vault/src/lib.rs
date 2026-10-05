mod backup;
mod lockout;
mod manager;
mod store;

pub use backup::{
    export_backup, export_payload, import_backup, is_backup, merge, BackupPayload, MergeOutcome,
};
pub use lockout::Lockout;
pub use manager::{MaintenanceReport, VaultManager};
pub use store::{IntegrityReport, Vault, TRASH_RETENTION_SECONDS};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum VaultError {
    #[error(transparent)]
    Crypto(#[from] liauth_crypto::CryptoError),
    #[error("io failure: {0}")]
    Io(#[from] std::io::Error),
    #[error("vault is locked")]
    Locked,
    #[error("vault not found")]
    NotFound,
    #[error("vault already exists")]
    AlreadyExists,
    #[error("unknown key slot {0}")]
    UnknownSlot(String),
    #[error("malformed vault: {0}")]
    Malformed(String),
}
