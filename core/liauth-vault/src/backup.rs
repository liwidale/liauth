use liauth_core::base32;
use liauth_core::{Account, Category};
use liauth_crypto::{open_envelope, seal_envelope, Envelope, KdfParams};
use serde::{Deserialize, Serialize};

use crate::store::Vault;
use crate::VaultError;

const PURPOSE: &str = "liauth.backup";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupPayload {
    #[serde(default)]
    pub accounts: Vec<Account>,
    #[serde(default)]
    pub categories: Vec<Category>,
    pub exported_at: i64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MergeOutcome {
    pub added_accounts: u32,
    pub added_categories: u32,
    pub skipped: u32,
}

pub fn export_backup(vault: &Vault, password: &str, now: i64) -> Result<Vec<u8>, VaultError> {
    export_payload(
        &BackupPayload {
            accounts: vault.accounts.clone(),
            categories: vault.categories.clone(),
            exported_at: now,
        },
        password,
    )
}

pub fn export_payload(payload: &BackupPayload, password: &str) -> Result<Vec<u8>, VaultError> {
    let plaintext = serde_json::to_vec(payload).map_err(|e| VaultError::Malformed(e.to_string()))?;
    let (envelope, _) = seal_envelope(password.as_bytes(), &plaintext, PURPOSE, KdfParams::default())?;
    Ok(envelope.to_bytes())
}

pub fn import_backup(bytes: &[u8], password: &str) -> Result<BackupPayload, VaultError> {
    let envelope = Envelope::from_bytes(bytes)?;
    if envelope.header.purpose != PURPOSE {
        return Err(VaultError::Malformed("not a backup file".into()));
    }
    let (plaintext, _) = open_envelope(password.as_bytes(), &envelope)?;
    serde_json::from_slice(&plaintext).map_err(|e| VaultError::Malformed(e.to_string()))
}

pub fn is_backup(bytes: &[u8]) -> bool {
    Envelope::from_bytes(bytes)
        .map(|e| e.header.purpose == PURPOSE)
        .unwrap_or(false)
}

pub fn merge(vault: &mut Vault, payload: BackupPayload) -> MergeOutcome {
    let mut outcome = MergeOutcome::default();

    for category in payload.categories {
        let exists = vault
            .categories
            .iter()
            .any(|c| c.id == category.id || c.name.eq_ignore_ascii_case(&category.name));
        if exists {
            outcome.skipped += 1;
        } else {
            vault.categories.push(category);
            outcome.added_categories += 1;
        }
    }

    for account in payload.accounts {
        let fingerprint = account_fingerprint(&account);
        let existing = vault
            .accounts
            .iter_mut()
            .find(|a| a.id == account.id || account_fingerprint(a) == fingerprint);
        if let Some(existing) = existing {
            // An incoming copy of a trashed account brings it back.
            if existing.is_deleted() && !account.is_deleted() {
                existing.deleted_at = None;
                outcome.added_accounts += 1;
            } else {
                outcome.skipped += 1;
            }
            continue;
        }
        let mut account = account;
        if let Some(category_id) = account.category_id {
            if vault.category(category_id).is_none() {
                account.category_id = None;
            }
        }
        vault.accounts.push(account);
        outcome.added_accounts += 1;
    }

    outcome
}

fn account_fingerprint(account: &Account) -> String {
    format!(
        "{}|{}|{}",
        base32::encode(&account.secret.0),
        account.issuer.to_lowercase(),
        account.name.to_lowercase()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use liauth_core::Account;

    #[test]
    fn backup_roundtrip() {
        let mut vault = Vault::default();
        vault
            .accounts
            .push(Account::new("GitHub".into(), "me".into(), b"secret".to_vec(), 0));
        vault.categories.push(Category::new("Finance".into(), 0));

        let bytes = export_backup(&vault, "backup pass", 100).unwrap();
        assert!(is_backup(&bytes));
        let payload = import_backup(&bytes, "backup pass").unwrap();
        assert_eq!(payload.accounts.len(), 1);
        assert_eq!(payload.categories.len(), 1);
        assert_eq!(payload.exported_at, 100);
    }

    #[test]
    fn wrong_password_rejected() {
        let vault = Vault::default();
        let bytes = export_backup(&vault, "backup pass", 0).unwrap();
        assert!(import_backup(&bytes, "other").is_err());
    }

    #[test]
    fn merge_deduplicates() {
        let mut vault = Vault::default();
        let account = Account::new("GitHub".into(), "me".into(), b"secret".to_vec(), 0);
        vault.accounts.push(account.clone());

        let payload = BackupPayload {
            accounts: vec![
                account,
                Account::new("GitLab".into(), "me".into(), b"other".to_vec(), 0),
            ],
            categories: vec![],
            exported_at: 0,
        };
        let outcome = merge(&mut vault, payload);
        assert_eq!(outcome.added_accounts, 1);
        assert_eq!(outcome.skipped, 1);
        assert_eq!(vault.accounts.len(), 2);
    }

    #[test]
    fn merge_drops_unknown_category_reference() {
        let mut vault = Vault::default();
        let mut account = Account::new("GitHub".into(), "me".into(), b"secret".to_vec(), 0);
        account.category_id = Some(uuid::Uuid::new_v4());
        let payload = BackupPayload {
            accounts: vec![account],
            categories: vec![],
            exported_at: 0,
        };
        merge(&mut vault, payload);
        assert_eq!(vault.accounts[0].category_id, None);
    }
}
