use thiserror::Error;

#[derive(Debug, Error, uniffi::Error)]
#[uniffi(flat_error)]
pub enum LiAuthError {
    #[error("wrong password")]
    WrongPassword,
    #[error("vault is locked")]
    Locked,
    #[error("vault not found")]
    VaultNotFound,
    #[error("vault already exists")]
    VaultAlreadyExists,
    #[error("item not found")]
    NotFound,
    #[error("invalid input: {message}")]
    InvalidInput { message: String },
    #[error("unrecognized import format")]
    UnrecognizedFormat,
    #[error("password required")]
    PasswordRequired,
    #[error("sync failure: {message}")]
    Sync { message: String },
    #[error("storage failure: {message}")]
    Storage { message: String },
    #[error("too many attempts, retry in {seconds} s")]
    RateLimited { seconds: u64 },
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct AccountView {
    pub id: String,
    pub issuer: String,
    pub name: String,
    pub is_counter_based: bool,
    pub category_id: Option<String>,
    pub pinned: bool,
    pub created_at: i64,
    pub algorithm: String,
    pub digits: u32,
    pub period: u32,
    pub notes: String,
    pub recovery_codes: Vec<String>,
}

/// An account waiting in the trash.
#[derive(Debug, Clone, uniffi::Record)]
pub struct TrashedAccountView {
    pub id: String,
    pub issuer: String,
    pub name: String,
    pub deleted_at: i64,
    /// Unix time when the entry will be purged automatically.
    pub purge_at: i64,
}

/// A fuzzy-search hit with highlight positions (character indices).
#[derive(Debug, Clone, uniffi::Record)]
pub struct SearchResultView {
    pub account: AccountView,
    pub issuer_indices: Vec<u32>,
    pub name_indices: Vec<u32>,
}

/// Outcome of the automatic startup check that runs on unlock.
#[derive(Debug, Clone, uniffi::Record)]
pub struct MaintenanceView {
    pub issues: Vec<String>,
    pub repaired: bool,
    pub purged_from_trash: u32,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct CodeView {
    pub id: String,
    pub code: String,
    pub seconds_remaining: u32,
    pub period: u32,
    pub is_counter_based: bool,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct CategoryView {
    pub id: String,
    pub name: String,
    pub position: u32,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct ImportSummary {
    pub source: String,
    pub added_accounts: u32,
    pub added_categories: u32,
    pub skipped: u32,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SyncSession {
    pub code: String,
    pub port: u16,
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum SyncReceiveStatus {
    Waiting,
    Completed {
        added_accounts: u32,
        added_categories: u32,
        skipped: u32,
    },
    Failed {
        message: String,
    },
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SyncPeerView {
    pub name: String,
    pub addresses: Vec<String>,
    pub port: u16,
}
