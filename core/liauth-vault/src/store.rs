use std::collections::{BTreeMap, HashSet};

use liauth_core::{Account, Category};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// How long a trashed account is kept before it is purged for good.
pub const TRASH_RETENTION_SECONDS: i64 = 30 * 24 * 60 * 60;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vault {
    #[serde(default)]
    pub accounts: Vec<Account>,
    #[serde(default)]
    pub categories: Vec<Category>,
    #[serde(default)]
    pub settings: BTreeMap<String, String>,
}

/// Result of the startup integrity check: what was wrong, whether it could
/// be repaired in place.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IntegrityReport {
    pub issues: Vec<String>,
    pub repaired: bool,
}

impl IntegrityReport {
    pub fn is_clean(&self) -> bool {
        self.issues.is_empty()
    }
}

impl Vault {
    pub fn account(&self, id: Uuid) -> Option<&Account> {
        self.accounts.iter().find(|a| a.id == id)
    }

    pub fn account_mut(&mut self, id: Uuid) -> Option<&mut Account> {
        self.accounts.iter_mut().find(|a| a.id == id)
    }

    /// Accounts that are not in the trash.
    pub fn active_accounts(&self) -> impl Iterator<Item = &Account> {
        self.accounts.iter().filter(|a| !a.is_deleted())
    }

    /// Accounts waiting in the trash, newest deletion first.
    pub fn trashed_accounts(&self) -> Vec<&Account> {
        let mut trashed: Vec<&Account> = self.accounts.iter().filter(|a| a.is_deleted()).collect();
        trashed.sort_by_key(|a| std::cmp::Reverse(a.deleted_at));
        trashed
    }

    /// Moves an account to the trash. Returns false when the id is unknown
    /// or the account is already trashed.
    pub fn trash_account(&mut self, id: Uuid, now: i64) -> bool {
        match self.account_mut(id) {
            Some(account) if !account.is_deleted() => {
                account.deleted_at = Some(now);
                account.pinned = false;
                account.updated_at = now;
                true
            }
            _ => false,
        }
    }

    /// Brings a trashed account back.
    pub fn restore_account(&mut self, id: Uuid, now: i64) -> bool {
        match self.account_mut(id) {
            Some(account) if account.is_deleted() => {
                account.deleted_at = None;
                account.updated_at = now;
                true
            }
            _ => false,
        }
    }

    /// Permanently removes an account, bypassing the trash.
    pub fn remove_account(&mut self, id: Uuid) -> bool {
        let before = self.accounts.len();
        self.accounts.retain(|a| a.id != id);
        self.accounts.len() != before
    }

    /// Drops every trashed account older than the retention window.
    /// Returns how many were purged.
    pub fn purge_expired(&mut self, now: i64) -> u32 {
        let before = self.accounts.len();
        self.accounts.retain(|a| match a.deleted_at {
            Some(deleted_at) => now - deleted_at < TRASH_RETENTION_SECONDS,
            None => true,
        });
        (before - self.accounts.len()) as u32
    }

    pub fn category(&self, id: Uuid) -> Option<&Category> {
        self.categories.iter().find(|c| c.id == id)
    }

    pub fn remove_category(&mut self, id: Uuid) -> bool {
        let before = self.categories.len();
        self.categories.retain(|c| c.id != id);
        if self.categories.len() == before {
            return false;
        }
        for account in &mut self.accounts {
            if account.category_id == Some(id) {
                account.category_id = None;
            }
        }
        true
    }

    /// Validates the vault structure and repairs what can be repaired:
    /// duplicate ids, dangling category references, empty secrets and
    /// out-of-range OTP parameters are detected; all but empty secrets are
    /// fixed in place.
    pub fn check_integrity(&mut self) -> IntegrityReport {
        let mut report = IntegrityReport::default();

        let mut seen = HashSet::new();
        let mut duplicates = 0u32;
        for account in &mut self.accounts {
            if !seen.insert(account.id) {
                account.id = Uuid::new_v4();
                duplicates += 1;
            }
        }
        if duplicates > 0 {
            report
                .issues
                .push(format!("{duplicates} duplicate account id(s)"));
            report.repaired = true;
        }

        let category_ids: HashSet<Uuid> = self.categories.iter().map(|c| c.id).collect();
        let mut dangling = 0u32;
        for account in &mut self.accounts {
            if let Some(category_id) = account.category_id {
                if !category_ids.contains(&category_id) {
                    account.category_id = None;
                    dangling += 1;
                }
            }
        }
        if dangling > 0 {
            report
                .issues
                .push(format!("{dangling} dangling category reference(s)"));
            report.repaired = true;
        }

        let mut bad_params = 0u32;
        for account in &mut self.accounts {
            if !(4..=10).contains(&account.digits) {
                account.digits = 6;
                bad_params += 1;
            }
            if let liauth_core::TokenKind::Totp { period } = account.kind {
                if !(5..=300).contains(&period) {
                    account.kind = liauth_core::TokenKind::Totp { period: 30 };
                    bad_params += 1;
                }
            }
        }
        if bad_params > 0 {
            report
                .issues
                .push(format!("{bad_params} out-of-range OTP parameter(s)"));
            report.repaired = true;
        }

        let empty_secrets = self.accounts.iter().filter(|a| a.secret.0.is_empty()).count();
        if empty_secrets > 0 {
            report
                .issues
                .push(format!("{empty_secrets} account(s) with an empty secret"));
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn account(issuer: &str) -> Account {
        Account::new(issuer.into(), "me".into(), b"secret".to_vec(), 0)
    }

    #[test]
    fn trash_restore_roundtrip() {
        let mut vault = Vault::default();
        let a = account("GitHub");
        let id = a.id;
        vault.accounts.push(a);

        assert!(vault.trash_account(id, 100));
        assert_eq!(vault.active_accounts().count(), 0);
        assert_eq!(vault.trashed_accounts().len(), 1);
        assert!(!vault.trash_account(id, 100));

        assert!(vault.restore_account(id, 200));
        assert_eq!(vault.active_accounts().count(), 1);
        assert!(vault.trashed_accounts().is_empty());
    }

    #[test]
    fn purge_respects_retention() {
        let mut vault = Vault::default();
        let a = account("Old");
        let b = account("Fresh");
        let (old_id, fresh_id) = (a.id, b.id);
        vault.accounts.push(a);
        vault.accounts.push(b);
        vault.trash_account(old_id, 0);
        vault.trash_account(fresh_id, TRASH_RETENTION_SECONDS - 10);

        let purged = vault.purge_expired(TRASH_RETENTION_SECONDS + 1);
        assert_eq!(purged, 1);
        assert!(vault.account(old_id).is_none());
        assert!(vault.account(fresh_id).is_some());
    }

    #[test]
    fn integrity_repairs_dangling_and_duplicates() {
        let mut vault = Vault::default();
        let mut a = account("A");
        a.category_id = Some(Uuid::new_v4());
        let mut b = account("B");
        b.id = a.id;
        b.digits = 42;
        vault.accounts.push(a);
        vault.accounts.push(b);

        let report = vault.check_integrity();
        assert!(!report.is_clean());
        assert!(report.repaired);
        assert_ne!(vault.accounts[0].id, vault.accounts[1].id);
        assert_eq!(vault.accounts[0].category_id, None);
        assert_eq!(vault.accounts[1].digits, 6);

        let second = vault.check_integrity();
        assert!(second.is_clean());
    }
}
