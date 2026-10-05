use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use liauth_core::time::{set_time_offset, system_now, time_offset, unix_now};
use liauth_core::{base32, otp, search, sntp, uri, Account, Algorithm, Category, TokenKind};
use liauth_crypto::KdfParams;
use liauth_import::{detect_and_import, ImportError, ImportSource};
use liauth_sync::webdav::{self, WebDavConfig};
use liauth_sync::{discover, send_payload, Receiver, ReceiverEvent};
use liauth_vault::{
    export_backup, import_backup, is_backup, merge, BackupPayload, Lockout, VaultError, VaultManager,
    TRASH_RETENTION_SECONDS,
};
use uuid::Uuid;

use crate::types::*;

const SETTING_AUTO_BACKUP_DIR: &str = "autoBackup.dir";
const SETTING_WEBDAV_URL: &str = "webdav.url";
const SETTING_WEBDAV_USER: &str = "webdav.username";
const SETTING_WEBDAV_PASSWORD: &str = "webdav.password";
const SETTING_WEBDAV_BACKUP_PASSWORD: &str = "webdav.backupPassword";
const WEBDAV_BACKUP_FILE: &str = "LiAuth-backup.liauthbackup";

struct State {
    manager: Option<VaultManager>,
    sync_receiver: Option<Receiver>,
}

#[derive(uniffi::Object)]
pub struct LiAuthEngine {
    path: PathBuf,
    kdf: KdfParams,
    state: Mutex<State>,
}

#[uniffi::export]
impl LiAuthEngine {
    #[uniffi::constructor]
    pub fn new(vault_path: String) -> Arc<Self> {
        Arc::new(Self {
            path: PathBuf::from(vault_path),
            kdf: KdfParams::default(),
            state: Mutex::new(State {
                manager: None,
                sync_receiver: None,
            }),
        })
    }

    #[uniffi::constructor]
    pub fn new_mobile(vault_path: String) -> Arc<Self> {
        Arc::new(Self {
            path: PathBuf::from(vault_path),
            kdf: KdfParams::mobile(),
            state: Mutex::new(State {
                manager: None,
                sync_receiver: None,
            }),
        })
    }

    pub fn vault_exists(&self) -> bool {
        VaultManager::exists(&self.path)
    }

    pub fn create_vault(&self, password: String) -> Result<(), LiAuthError> {
        let manager = VaultManager::create(&self.path, &password, self.kdf.clone())?;
        self.state.lock().unwrap().manager = Some(manager);
        Ok(())
    }

    pub fn unlock(&self, password: String) -> Result<MaintenanceView, LiAuthError> {
        let mut lockout = Lockout::for_vault(&self.path);
        let wait = lockout.remaining_delay(system_now());
        if wait > 0 {
            return Err(LiAuthError::RateLimited { seconds: wait });
        }
        let mut manager = VaultManager::open(&self.path)?;
        match manager.unlock_with_password(&password) {
            Ok(()) => lockout.record_success(),
            Err(error) => {
                let error = LiAuthError::from(error);
                if matches!(error, LiAuthError::WrongPassword) {
                    lockout.record_failure(system_now());
                }
                return Err(error);
            }
        }
        self.finish_unlock(manager)
    }

    pub fn unlock_with_slot(&self, slot: String, key: Vec<u8>) -> Result<MaintenanceView, LiAuthError> {
        let mut manager = VaultManager::open(&self.path)?;
        manager.unlock_with_slot(&slot, &key)?;
        Lockout::for_vault(&self.path).record_success();
        self.finish_unlock(manager)
    }

    /// Seconds the user still has to wait before the next password attempt.
    pub fn lockout_remaining_seconds(&self) -> u64 {
        Lockout::for_vault(&self.path).remaining_delay(system_now())
    }

    pub fn add_key_slot(&self, slot: String, key: Vec<u8>) -> Result<(), LiAuthError> {
        self.with_manager(|manager| Ok(manager.add_key_slot(&slot, &key)?))
    }

    pub fn remove_key_slot(&self, slot: String) -> Result<(), LiAuthError> {
        self.with_manager(|manager| Ok(manager.remove_key_slot(&slot)?))
    }

    pub fn has_key_slot(&self, slot: String) -> bool {
        if let Ok(manager) = VaultManager::open(&self.path) {
            return manager.has_key_slot(&slot);
        }
        false
    }

    pub fn lock(&self) {
        let mut state = self.state.lock().unwrap();
        if let Some(manager) = state.manager.as_mut() {
            manager.lock();
        }
        state.manager = None;
        if let Some(mut receiver) = state.sync_receiver.take() {
            receiver.stop();
        }
    }

    pub fn is_unlocked(&self) -> bool {
        self.state
            .lock()
            .unwrap()
            .manager
            .as_ref()
            .map(|m| m.is_unlocked())
            .unwrap_or(false)
    }

    pub fn verify_password(&self, password: String) -> bool {
        VaultManager::open(&self.path)
            .map(|m| m.verify_password(&password))
            .unwrap_or(false)
    }

    pub fn change_password(&self, current: String, new: String) -> Result<(), LiAuthError> {
        let kdf = self.kdf.clone();
        self.with_manager(move |manager| Ok(manager.change_password(&current, &new, kdf)?))
    }

    pub fn accounts(&self) -> Result<Vec<AccountView>, LiAuthError> {
        self.with_manager(|manager| {
            let vault = manager.vault()?;
            let mut accounts: Vec<&Account> = vault.active_accounts().collect();
            accounts.sort_by(|a, b| {
                b.pinned.cmp(&a.pinned).then_with(|| {
                    a.display_title()
                        .to_lowercase()
                        .cmp(&b.display_title().to_lowercase())
                })
            });
            Ok(accounts.into_iter().map(account_view).collect())
        })
    }

    /// Typo-tolerant search over issuer and account name. Results carry the
    /// matched character positions so UIs can highlight them.
    pub fn search_accounts(&self, query: String) -> Result<Vec<SearchResultView>, LiAuthError> {
        self.with_manager(move |manager| {
            let vault = manager.vault()?;
            let mut hits: Vec<(i64, SearchResultView)> = vault
                .active_accounts()
                .filter_map(|account| {
                    search::match_account(account, &query).map(|m| {
                        (
                            m.score,
                            SearchResultView {
                                account: account_view(account),
                                issuer_indices: m.issuer_indices,
                                name_indices: m.name_indices,
                            },
                        )
                    })
                })
                .collect();
            hits.sort_by(|a, b| {
                b.0.cmp(&a.0).then_with(|| {
                    b.1.account.pinned.cmp(&a.1.account.pinned).then_with(|| {
                        a.1.account
                            .issuer
                            .to_lowercase()
                            .cmp(&b.1.account.issuer.to_lowercase())
                    })
                })
            });
            Ok(hits.into_iter().map(|(_, view)| view).collect())
        })
    }

    pub fn codes(&self) -> Result<Vec<CodeView>, LiAuthError> {
        self.with_manager(|manager| {
            let vault = manager.vault()?;
            let now = unix_now();
            Ok(vault
                .active_accounts()
                .map(|account| code_view(account, now))
                .collect())
        })
    }

    pub fn add_account_uri(&self, uri_value: String) -> Result<AccountView, LiAuthError> {
        let account = uri::parse(uri_value.trim()).map_err(|e| LiAuthError::InvalidInput {
            message: e.to_string(),
        })?;
        self.insert_account(account)
    }

    pub fn add_account_manual(
        &self,
        issuer: String,
        name: String,
        secret: String,
    ) -> Result<AccountView, LiAuthError> {
        let trimmed = secret.trim();
        if trimmed.starts_with("otpauth://") {
            return self.add_account_uri(secret);
        }
        let decoded = base32::decode(trimmed).map_err(|_| LiAuthError::InvalidInput {
            message: "invalid secret key".into(),
        })?;
        let account = Account::new(
            issuer.trim().to_string(),
            name.trim().to_string(),
            decoded,
            unix_now(),
        );
        self.insert_account(account)
    }

    /// Manual entry with non-default TOTP parameters (8 digits, SHA-256/512,
    /// custom period).
    pub fn add_account_manual_advanced(
        &self,
        issuer: String,
        name: String,
        secret: String,
        algorithm: String,
        digits: u32,
        period: u32,
    ) -> Result<AccountView, LiAuthError> {
        let algorithm = Algorithm::parse(&algorithm).ok_or_else(|| LiAuthError::InvalidInput {
            message: "unknown algorithm".into(),
        })?;
        if !(4..=10).contains(&digits) {
            return Err(LiAuthError::InvalidInput {
                message: "digits out of range".into(),
            });
        }
        if !(5..=300).contains(&period) {
            return Err(LiAuthError::InvalidInput {
                message: "period out of range".into(),
            });
        }
        let decoded = base32::decode(secret.trim()).map_err(|_| LiAuthError::InvalidInput {
            message: "invalid secret key".into(),
        })?;
        let mut account = Account::new(
            issuer.trim().to_string(),
            name.trim().to_string(),
            decoded,
            unix_now(),
        );
        account.algorithm = algorithm;
        account.digits = digits;
        account.kind = TokenKind::Totp { period };
        self.insert_account(account)
    }

    pub fn update_account(
        &self,
        id: String,
        issuer: String,
        name: String,
        category_id: Option<String>,
        pinned: bool,
    ) -> Result<(), LiAuthError> {
        let account_id = parse_id(&id)?;
        let category_uuid = category_id.as_deref().map(parse_id).transpose()?;
        self.with_manager(move |manager| {
            let vault = manager.vault_mut()?;
            let account = vault.account_mut(account_id).ok_or(LiAuthError::NotFound)?;
            account.issuer = issuer.trim().to_string();
            account.name = name.trim().to_string();
            account.category_id = category_uuid;
            account.pinned = pinned;
            account.updated_at = unix_now();
            manager.save()?;
            Ok(())
        })
    }

    pub fn update_account_advanced(
        &self,
        id: String,
        algorithm: String,
        digits: u32,
        period: u32,
    ) -> Result<(), LiAuthError> {
        let account_id = parse_id(&id)?;
        let algorithm = Algorithm::parse(&algorithm).ok_or_else(|| LiAuthError::InvalidInput {
            message: "unknown algorithm".into(),
        })?;
        if !(4..=10).contains(&digits) {
            return Err(LiAuthError::InvalidInput {
                message: "digits out of range".into(),
            });
        }
        if !(5..=300).contains(&period) {
            return Err(LiAuthError::InvalidInput {
                message: "period out of range".into(),
            });
        }
        self.with_manager(move |manager| {
            let vault = manager.vault_mut()?;
            let account = vault.account_mut(account_id).ok_or(LiAuthError::NotFound)?;
            account.algorithm = algorithm;
            account.digits = digits;
            if let TokenKind::Totp { .. } = account.kind {
                account.kind = TokenKind::Totp { period };
            }
            account.updated_at = unix_now();
            manager.save()?;
            Ok(())
        })
    }

    pub fn advance_counter(&self, id: String) -> Result<CodeView, LiAuthError> {
        let account_id = parse_id(&id)?;
        self.with_manager(move |manager| {
            let vault = manager.vault_mut()?;
            let account = vault.account_mut(account_id).ok_or(LiAuthError::NotFound)?;
            if let TokenKind::Hotp { counter } = account.kind {
                account.kind = TokenKind::Hotp { counter: counter + 1 };
                account.updated_at = unix_now();
            }
            let view = code_view(account, unix_now());
            manager.save()?;
            Ok(view)
        })
    }

    /// Moves an account to the trash, where it is kept for 30 days.
    pub fn delete_account(&self, id: String) -> Result<(), LiAuthError> {
        let account_id = parse_id(&id)?;
        self.with_manager(move |manager| {
            if !manager.vault_mut()?.trash_account(account_id, unix_now()) {
                return Err(LiAuthError::NotFound);
            }
            manager.save()?;
            Ok(())
        })
    }

    /// Moves several accounts to the trash at once.
    pub fn delete_accounts(&self, ids: Vec<String>) -> Result<u32, LiAuthError> {
        let account_ids: Vec<Uuid> = ids.iter().map(|id| parse_id(id)).collect::<Result<_, _>>()?;
        self.with_manager(move |manager| {
            let vault = manager.vault_mut()?;
            let now = unix_now();
            let mut trashed = 0u32;
            for account_id in account_ids {
                if vault.trash_account(account_id, now) {
                    trashed += 1;
                }
            }
            if trashed > 0 {
                manager.save()?;
            }
            Ok(trashed)
        })
    }

    /// Assigns several accounts to a category (or clears it) at once.
    pub fn set_accounts_category(
        &self,
        ids: Vec<String>,
        category_id: Option<String>,
    ) -> Result<u32, LiAuthError> {
        let account_ids: Vec<Uuid> = ids.iter().map(|id| parse_id(id)).collect::<Result<_, _>>()?;
        let category_uuid = category_id.as_deref().map(parse_id).transpose()?;
        self.with_manager(move |manager| {
            let vault = manager.vault_mut()?;
            if let Some(category_uuid) = category_uuid {
                if vault.category(category_uuid).is_none() {
                    return Err(LiAuthError::NotFound);
                }
            }
            let now = unix_now();
            let mut moved = 0u32;
            for account_id in account_ids {
                if let Some(account) = vault.account_mut(account_id) {
                    account.category_id = category_uuid;
                    account.updated_at = now;
                    moved += 1;
                }
            }
            if moved > 0 {
                manager.save()?;
            }
            Ok(moved)
        })
    }

    pub fn trashed_accounts(&self) -> Result<Vec<TrashedAccountView>, LiAuthError> {
        self.with_manager(|manager| {
            Ok(manager
                .vault()?
                .trashed_accounts()
                .into_iter()
                .map(|account| {
                    let deleted_at = account.deleted_at.unwrap_or(0);
                    TrashedAccountView {
                        id: account.id.to_string(),
                        issuer: account.issuer.clone(),
                        name: account.name.clone(),
                        deleted_at,
                        purge_at: deleted_at + TRASH_RETENTION_SECONDS,
                    }
                })
                .collect())
        })
    }

    pub fn restore_account(&self, id: String) -> Result<(), LiAuthError> {
        let account_id = parse_id(&id)?;
        self.with_manager(move |manager| {
            if !manager.vault_mut()?.restore_account(account_id, unix_now()) {
                return Err(LiAuthError::NotFound);
            }
            manager.save()?;
            Ok(())
        })
    }

    /// Permanently removes an account, bypassing or emptying the trash.
    pub fn purge_account(&self, id: String) -> Result<(), LiAuthError> {
        let account_id = parse_id(&id)?;
        self.with_manager(move |manager| {
            if !manager.vault_mut()?.remove_account(account_id) {
                return Err(LiAuthError::NotFound);
            }
            manager.save()?;
            Ok(())
        })
    }

    /// Stores free-form notes and recovery codes alongside an account.
    pub fn update_account_notes(
        &self,
        id: String,
        notes: String,
        recovery_codes: Vec<String>,
    ) -> Result<(), LiAuthError> {
        let account_id = parse_id(&id)?;
        self.with_manager(move |manager| {
            let vault = manager.vault_mut()?;
            let account = vault.account_mut(account_id).ok_or(LiAuthError::NotFound)?;
            account.notes = notes;
            account.recovery_codes = recovery_codes
                .into_iter()
                .map(|c| c.trim().to_string())
                .filter(|c| !c.is_empty())
                .collect();
            account.updated_at = unix_now();
            manager.save()?;
            Ok(())
        })
    }

    pub fn account_uri(&self, id: String) -> Result<String, LiAuthError> {
        let account_id = parse_id(&id)?;
        self.with_manager(move |manager| {
            let vault = manager.vault()?;
            let account = vault.account(account_id).ok_or(LiAuthError::NotFound)?;
            Ok(uri::build(account))
        })
    }

    pub fn categories(&self) -> Result<Vec<CategoryView>, LiAuthError> {
        self.with_manager(|manager| {
            let vault = manager.vault()?;
            let mut categories: Vec<&Category> = vault.categories.iter().collect();
            categories.sort_by_key(|c| c.position);
            Ok(categories
                .into_iter()
                .map(|c| CategoryView {
                    id: c.id.to_string(),
                    name: c.name.clone(),
                    position: c.position,
                })
                .collect())
        })
    }

    pub fn add_category(&self, name: String) -> Result<CategoryView, LiAuthError> {
        let trimmed = name.trim().to_string();
        if trimmed.is_empty() {
            return Err(LiAuthError::InvalidInput {
                message: "empty category name".into(),
            });
        }
        self.with_manager(move |manager| {
            let vault = manager.vault_mut()?;
            let position = vault.categories.iter().map(|c| c.position + 1).max().unwrap_or(0);
            let category = Category::new(trimmed, position);
            let view = CategoryView {
                id: category.id.to_string(),
                name: category.name.clone(),
                position: category.position,
            };
            vault.categories.push(category);
            manager.save()?;
            Ok(view)
        })
    }

    pub fn rename_category(&self, id: String, name: String) -> Result<(), LiAuthError> {
        let category_id = parse_id(&id)?;
        let trimmed = name.trim().to_string();
        if trimmed.is_empty() {
            return Err(LiAuthError::InvalidInput {
                message: "empty category name".into(),
            });
        }
        self.with_manager(move |manager| {
            let vault = manager.vault_mut()?;
            let category = vault
                .categories
                .iter_mut()
                .find(|c| c.id == category_id)
                .ok_or(LiAuthError::NotFound)?;
            category.name = trimmed;
            manager.save()?;
            Ok(())
        })
    }

    pub fn delete_category(&self, id: String) -> Result<(), LiAuthError> {
        let category_id = parse_id(&id)?;
        self.with_manager(move |manager| {
            if !manager.vault_mut()?.remove_category(category_id) {
                return Err(LiAuthError::NotFound);
            }
            manager.save()?;
            Ok(())
        })
    }

    pub fn get_setting(&self, key: String) -> Result<Option<String>, LiAuthError> {
        self.with_manager(move |manager| Ok(manager.vault()?.settings.get(&key).cloned()))
    }

    pub fn set_setting(&self, key: String, value: String) -> Result<(), LiAuthError> {
        self.with_manager(move |manager| {
            manager.vault_mut()?.settings.insert(key, value);
            manager.save()?;
            Ok(())
        })
    }

    /// Measures the clock drift against public NTP servers and applies it to
    /// code generation. The OS clock itself is never modified. Returns the
    /// measured offset in seconds.
    pub fn sync_time_drift(&self) -> Result<i64, LiAuthError> {
        let offset = sntp::measure_offset(&sntp::DEFAULT_SERVERS, Duration::from_secs(5)).map_err(|e| {
            LiAuthError::Sync {
                message: e.to_string(),
            }
        })?;
        set_time_offset(offset);
        Ok(offset)
    }

    /// The drift correction currently applied, in seconds.
    pub fn time_drift_seconds(&self) -> i64 {
        time_offset()
    }

    /// Enables (or disables, with None) the automatic encrypted copy of the
    /// vault written to an external directory after every change.
    pub fn set_auto_backup_dir(&self, dir: Option<String>) -> Result<(), LiAuthError> {
        self.with_manager(move |manager| {
            let dir = dir.filter(|d| !d.trim().is_empty());
            match dir.as_ref() {
                Some(value) => {
                    manager
                        .vault_mut()?
                        .settings
                        .insert(SETTING_AUTO_BACKUP_DIR.into(), value.clone());
                }
                None => {
                    manager.vault_mut()?.settings.remove(SETTING_AUTO_BACKUP_DIR);
                }
            }
            manager.set_auto_backup_dir(dir.map(PathBuf::from));
            manager.save()?;
            Ok(())
        })
    }

    pub fn auto_backup_dir(&self) -> Result<Option<String>, LiAuthError> {
        self.with_manager(|manager| Ok(manager.vault()?.settings.get(SETTING_AUTO_BACKUP_DIR).cloned()))
    }

    /// Checks that a WebDAV endpoint is reachable with these credentials.
    pub fn webdav_test(&self, url: String, username: String, password: String) -> Result<(), LiAuthError> {
        webdav::check_connection(&WebDavConfig {
            url,
            username,
            password,
        })
        .map_err(|e| LiAuthError::Sync {
            message: e.to_string(),
        })
    }

    /// Stores the WebDAV target inside the encrypted vault and uploads a
    /// first backup. Pass an empty url to disable.
    pub fn webdav_configure(
        &self,
        url: String,
        username: String,
        password: String,
        backup_password: String,
    ) -> Result<(), LiAuthError> {
        self.with_manager(|manager| {
            let settings = &mut manager.vault_mut()?.settings;
            if url.trim().is_empty() {
                settings.remove(SETTING_WEBDAV_URL);
                settings.remove(SETTING_WEBDAV_USER);
                settings.remove(SETTING_WEBDAV_PASSWORD);
                settings.remove(SETTING_WEBDAV_BACKUP_PASSWORD);
            } else {
                if backup_password.is_empty() {
                    return Err(LiAuthError::InvalidInput {
                        message: "backup password required".into(),
                    });
                }
                settings.insert(SETTING_WEBDAV_URL.into(), url.trim().to_string());
                settings.insert(SETTING_WEBDAV_USER.into(), username);
                settings.insert(SETTING_WEBDAV_PASSWORD.into(), password);
                settings.insert(SETTING_WEBDAV_BACKUP_PASSWORD.into(), backup_password);
            }
            manager.save()?;
            Ok(())
        })?;
        if self.webdav_is_configured() {
            self.webdav_sync_now()?;
        }
        Ok(())
    }

    pub fn webdav_is_configured(&self) -> bool {
        self.with_manager(|manager| {
            Ok(manager
                .vault()?
                .settings
                .get(SETTING_WEBDAV_URL)
                .map(|u| !u.is_empty())
                .unwrap_or(false))
        })
        .unwrap_or(false)
    }

    /// Exports an encrypted backup and uploads it to the configured WebDAV
    /// share. Call after changes (from a background thread) to keep the
    /// remote copy fresh.
    pub fn webdav_sync_now(&self) -> Result<(), LiAuthError> {
        let (config, backup) = self.with_manager(|manager| {
            let vault = manager.vault()?;
            let url = vault
                .settings
                .get(SETTING_WEBDAV_URL)
                .cloned()
                .unwrap_or_default();
            if url.is_empty() {
                return Err(LiAuthError::InvalidInput {
                    message: "webdav is not configured".into(),
                });
            }
            let config = WebDavConfig {
                url,
                username: vault
                    .settings
                    .get(SETTING_WEBDAV_USER)
                    .cloned()
                    .unwrap_or_default(),
                password: vault
                    .settings
                    .get(SETTING_WEBDAV_PASSWORD)
                    .cloned()
                    .unwrap_or_default(),
            };
            let backup_password = vault
                .settings
                .get(SETTING_WEBDAV_BACKUP_PASSWORD)
                .cloned()
                .unwrap_or_default();
            let backup = export_backup(vault, &backup_password, unix_now())?;
            Ok((config, backup))
        })?;
        webdav::upload(&config, WEBDAV_BACKUP_FILE, &backup).map_err(|e| LiAuthError::Sync {
            message: e.to_string(),
        })
    }

    /// Downloads the backup stored on the configured WebDAV share and merges
    /// it into the vault.
    pub fn webdav_restore(&self) -> Result<ImportSummary, LiAuthError> {
        let config = self.with_manager(|manager| {
            let vault = manager.vault()?;
            let url = vault
                .settings
                .get(SETTING_WEBDAV_URL)
                .cloned()
                .unwrap_or_default();
            if url.is_empty() {
                return Err(LiAuthError::InvalidInput {
                    message: "webdav is not configured".into(),
                });
            }
            Ok(WebDavConfig {
                url,
                username: vault
                    .settings
                    .get(SETTING_WEBDAV_USER)
                    .cloned()
                    .unwrap_or_default(),
                password: vault
                    .settings
                    .get(SETTING_WEBDAV_PASSWORD)
                    .cloned()
                    .unwrap_or_default(),
            })
        })?;
        let bytes = webdav::download(&config, WEBDAV_BACKUP_FILE).map_err(|e| LiAuthError::Sync {
            message: e.to_string(),
        })?;
        let backup_password = self.with_manager(|manager| {
            Ok(manager
                .vault()?
                .settings
                .get(SETTING_WEBDAV_BACKUP_PASSWORD)
                .cloned())
        })?;
        self.import_data(bytes, backup_password)
    }

    pub fn export_backup(&self, password: String) -> Result<Vec<u8>, LiAuthError> {
        self.with_manager(move |manager| Ok(export_backup(manager.vault()?, &password, unix_now())?))
    }

    pub fn import_data(&self, data: Vec<u8>, password: Option<String>) -> Result<ImportSummary, LiAuthError> {
        self.with_manager(move |manager| {
            if is_backup(&data) {
                let password = password.as_deref().ok_or(LiAuthError::PasswordRequired)?;
                let payload = import_backup(&data, password)?;
                let outcome = merge(manager.vault_mut()?, payload);
                manager.save()?;
                return Ok(ImportSummary {
                    source: "liauth".into(),
                    added_accounts: outcome.added_accounts,
                    added_categories: outcome.added_categories,
                    skipped: outcome.skipped,
                });
            }
            let result = detect_and_import(&data, password.as_deref()).map_err(import_error)?;
            let payload = BackupPayload {
                accounts: result.accounts,
                categories: vec![],
                exported_at: unix_now(),
            };
            let outcome = merge(manager.vault_mut()?, payload);
            manager.save()?;
            Ok(ImportSummary {
                source: source_name(result.source).to_string(),
                added_accounts: outcome.added_accounts,
                added_categories: outcome.added_categories,
                skipped: outcome.skipped,
            })
        })
    }

    pub fn sync_start_receiver(&self, device_name: String) -> Result<SyncSession, LiAuthError> {
        let mut state = self.state.lock().unwrap();
        if state.manager.as_ref().map(|m| m.is_unlocked()).unwrap_or(false) {
            let receiver = Receiver::start(&device_name).map_err(|e| LiAuthError::Sync {
                message: e.to_string(),
            })?;
            let session = SyncSession {
                code: receiver.code.clone(),
                port: receiver.port,
            };
            state.sync_receiver = Some(receiver);
            Ok(session)
        } else {
            Err(LiAuthError::Locked)
        }
    }

    pub fn sync_poll_receiver(&self) -> Result<SyncReceiveStatus, LiAuthError> {
        let mut state = self.state.lock().unwrap();
        let Some(receiver) = state.sync_receiver.as_ref() else {
            return Ok(SyncReceiveStatus::Waiting);
        };
        match receiver.poll(Duration::from_millis(50)) {
            None => Ok(SyncReceiveStatus::Waiting),
            Some(ReceiverEvent::Failed(error)) => {
                state.sync_receiver = None;
                Ok(SyncReceiveStatus::Failed {
                    message: error.to_string(),
                })
            }
            Some(ReceiverEvent::Payload(payload)) => {
                state.sync_receiver = None;
                let manager = state.manager.as_mut().ok_or(LiAuthError::Locked)?;
                let parsed: BackupPayload =
                    serde_json::from_slice(&payload).map_err(|e| LiAuthError::Sync {
                        message: e.to_string(),
                    })?;
                let outcome = merge(manager.vault_mut()?, parsed);
                manager.save()?;
                Ok(SyncReceiveStatus::Completed {
                    added_accounts: outcome.added_accounts,
                    added_categories: outcome.added_categories,
                    skipped: outcome.skipped,
                })
            }
        }
    }

    pub fn sync_stop_receiver(&self) {
        if let Some(mut receiver) = self.state.lock().unwrap().sync_receiver.take() {
            receiver.stop();
        }
    }

    pub fn sync_discover(&self, timeout_ms: u64) -> Result<Vec<SyncPeerView>, LiAuthError> {
        let peers = discover(Duration::from_millis(timeout_ms)).map_err(|e| LiAuthError::Sync {
            message: e.to_string(),
        })?;
        Ok(peers
            .into_iter()
            .map(|p| SyncPeerView {
                name: p.name,
                addresses: p.addresses.iter().map(|a| a.to_string()).collect(),
                port: p.port,
            })
            .collect())
    }

    pub fn sync_send(&self, addresses: Vec<String>, port: u16, code: String) -> Result<(), LiAuthError> {
        let payload = self.with_manager(|manager| {
            let vault = manager.vault()?;
            let payload = BackupPayload {
                accounts: vault.active_accounts().cloned().collect(),
                categories: vault.categories.clone(),
                exported_at: unix_now(),
            };
            serde_json::to_vec(&payload).map_err(|e| LiAuthError::Sync {
                message: e.to_string(),
            })
        })?;
        let ips: Vec<IpAddr> = addresses.iter().filter_map(|a| a.parse().ok()).collect();
        if ips.is_empty() {
            return Err(LiAuthError::InvalidInput {
                message: "invalid address".into(),
            });
        }
        send_payload(&ips, port, &code, &payload).map_err(|e| match e {
            liauth_sync::SyncError::PairingFailed => LiAuthError::WrongPassword,
            other => LiAuthError::Sync {
                message: other.to_string(),
            },
        })
    }
}

impl LiAuthEngine {
    /// Shared tail of every unlock path: configures the auto-backup target
    /// from settings and runs the startup integrity / trash-retention pass.
    fn finish_unlock(&self, mut manager: VaultManager) -> Result<MaintenanceView, LiAuthError> {
        let backup_dir = manager
            .vault()?
            .settings
            .get(SETTING_AUTO_BACKUP_DIR)
            .filter(|d| !d.is_empty())
            .cloned();
        manager.set_auto_backup_dir(backup_dir.map(PathBuf::from));
        let report = manager.startup_maintenance(unix_now())?;
        self.state.lock().unwrap().manager = Some(manager);
        Ok(MaintenanceView {
            issues: report.integrity.issues,
            repaired: report.integrity.repaired,
            purged_from_trash: report.purged_from_trash,
        })
    }

    fn with_manager<T>(
        &self,
        f: impl FnOnce(&mut VaultManager) -> Result<T, LiAuthError>,
    ) -> Result<T, LiAuthError> {
        let mut state = self.state.lock().unwrap();
        let manager = state.manager.as_mut().ok_or(LiAuthError::Locked)?;
        f(manager)
    }

    fn insert_account(&self, account: Account) -> Result<AccountView, LiAuthError> {
        self.with_manager(move |manager| {
            let view = account_view(&account);
            manager.vault_mut()?.accounts.push(account);
            manager.save()?;
            Ok(view)
        })
    }
}

fn account_view(account: &Account) -> AccountView {
    let (is_counter_based, period) = match account.kind {
        TokenKind::Hotp { .. } => (true, 0),
        TokenKind::Totp { period } => (false, period),
        TokenKind::Steam => (false, 30),
    };
    AccountView {
        id: account.id.to_string(),
        issuer: account.issuer.clone(),
        name: account.name.clone(),
        is_counter_based,
        category_id: account.category_id.map(|id| id.to_string()),
        pinned: account.pinned,
        created_at: account.created_at,
        algorithm: account.algorithm.name().to_string(),
        digits: account.digits,
        period,
        notes: account.notes.clone(),
        recovery_codes: account.recovery_codes.clone(),
    }
}

fn code_view(account: &Account, now: i64) -> CodeView {
    let code = otp::code_for(
        &account.secret.0,
        account.kind,
        account.algorithm,
        account.digits,
        now,
    );
    let (seconds_remaining, period, is_counter_based) = match account.kind {
        TokenKind::Totp { period } => (otp::seconds_remaining(now, period), period, false),
        TokenKind::Steam => (otp::seconds_remaining(now, 30), 30, false),
        TokenKind::Hotp { .. } => (0, 0, true),
    };
    CodeView {
        id: account.id.to_string(),
        code,
        seconds_remaining,
        period,
        is_counter_based,
    }
}

fn parse_id(value: &str) -> Result<Uuid, LiAuthError> {
    Uuid::parse_str(value).map_err(|_| LiAuthError::InvalidInput {
        message: "invalid id".into(),
    })
}

fn import_error(error: ImportError) -> LiAuthError {
    match error {
        ImportError::PasswordRequired => LiAuthError::PasswordRequired,
        ImportError::WrongPassword => LiAuthError::WrongPassword,
        ImportError::Unrecognized => LiAuthError::UnrecognizedFormat,
        ImportError::Empty => LiAuthError::UnrecognizedFormat,
        ImportError::Malformed(message) => LiAuthError::InvalidInput { message },
    }
}

fn source_name(source: ImportSource) -> &'static str {
    match source {
        ImportSource::LiAuthBackup => "liauth",
        ImportSource::GoogleAuthenticator => "google",
        ImportSource::Aegis => "aegis",
        ImportSource::TwoFas => "twofas",
        ImportSource::Authy => "authy",
        ImportSource::UriList => "uri",
    }
}

impl From<VaultError> for LiAuthError {
    fn from(error: VaultError) -> Self {
        match error {
            VaultError::Crypto(liauth_crypto::CryptoError::Unauthenticated) => Self::WrongPassword,
            VaultError::Locked => Self::Locked,
            VaultError::NotFound => Self::VaultNotFound,
            VaultError::AlreadyExists => Self::VaultAlreadyExists,
            VaultError::UnknownSlot(slot) => Self::InvalidInput {
                message: format!("unknown slot {slot}"),
            },
            other => Self::Storage {
                message: other.to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine() -> (tempfile::TempDir, Arc<LiAuthEngine>) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vault.liauth").to_string_lossy().into_owned();
        let engine = LiAuthEngine::new_mobile(path);
        (dir, engine)
    }

    #[test]
    fn full_lifecycle() {
        let (_dir, engine) = engine();
        assert!(!engine.vault_exists());
        engine.create_vault("password".into()).unwrap();
        assert!(engine.is_unlocked());

        let view = engine
            .add_account_uri("otpauth://totp/GitHub:me?secret=MZXW6YTBOI&issuer=GitHub".into())
            .unwrap();
        assert_eq!(view.issuer, "GitHub");

        let codes = engine.codes().unwrap();
        assert_eq!(codes.len(), 1);
        assert_eq!(codes[0].code.len(), 6);

        let category = engine.add_category("Development".into()).unwrap();
        engine
            .update_account(
                view.id.clone(),
                "GitHub".into(),
                "me".into(),
                Some(category.id.clone()),
                true,
            )
            .unwrap();
        let accounts = engine.accounts().unwrap();
        assert_eq!(accounts[0].category_id, Some(category.id));
        assert!(accounts[0].pinned);

        engine.lock();
        assert!(!engine.is_unlocked());
        engine.unlock("password".into()).unwrap();
        assert_eq!(engine.accounts().unwrap().len(), 1);
    }

    #[test]
    fn backup_roundtrip_through_engine() {
        let (_dir, engine) = engine();
        engine.create_vault("password".into()).unwrap();
        engine
            .add_account_manual("GitHub".into(), "me".into(), "MZXW6YTBOI".into())
            .unwrap();

        let backup = engine.export_backup("backup pass".into()).unwrap();
        engine
            .delete_account(engine.accounts().unwrap()[0].id.clone())
            .unwrap();
        assert!(engine.accounts().unwrap().is_empty());

        let summary = engine.import_data(backup, Some("backup pass".into())).unwrap();
        assert_eq!(summary.added_accounts, 1);
        assert_eq!(engine.accounts().unwrap().len(), 1);
    }

    #[test]
    fn hotp_counter_advances() {
        let (_dir, engine) = engine();
        engine.create_vault("password".into()).unwrap();
        let view = engine
            .add_account_uri(
                "otpauth://hotp/Vendor:me?secret=GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ&counter=0".into(),
            )
            .unwrap();
        let first = engine.codes().unwrap()[0].code.clone();
        let advanced = engine.advance_counter(view.id).unwrap();
        assert_ne!(first, advanced.code);
    }
}
