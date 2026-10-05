use std::fs;
use std::path::{Path, PathBuf};

use liauth_crypto::{
    open_envelope, open_with_key, reseal_with_key, seal_envelope, unwrap_key, wrap_key, Envelope, KdfParams,
    SymmetricKey,
};

use crate::store::{IntegrityReport, Vault};
use crate::VaultError;

const PURPOSE: &str = "liauth.vault";
const AUTO_BACKUP_FILE: &str = "LiAuth-auto-backup.liauth";

/// What the automatic startup pass did after an unlock.
#[derive(Debug, Clone, Default)]
pub struct MaintenanceReport {
    pub integrity: IntegrityReport,
    pub purged_from_trash: u32,
}

pub struct VaultManager {
    path: PathBuf,
    envelope: Envelope,
    data_key: Option<SymmetricKey>,
    vault: Option<Vault>,
    auto_backup_dir: Option<PathBuf>,
}

impl VaultManager {
    pub fn exists(path: &Path) -> bool {
        path.is_file()
    }

    pub fn create(path: &Path, password: &str, params: KdfParams) -> Result<Self, VaultError> {
        if Self::exists(path) {
            return Err(VaultError::AlreadyExists);
        }
        let vault = Vault::default();
        let plaintext = serde_json::to_vec(&vault).map_err(|e| VaultError::Malformed(e.to_string()))?;
        let (envelope, data_key) = seal_envelope(password.as_bytes(), &plaintext, PURPOSE, params)?;
        let manager = Self {
            path: path.to_path_buf(),
            envelope,
            data_key: Some(data_key),
            vault: Some(vault),
            auto_backup_dir: None,
        };
        manager.persist()?;
        Ok(manager)
    }

    pub fn open(path: &Path) -> Result<Self, VaultError> {
        if !Self::exists(path) {
            return Err(VaultError::NotFound);
        }
        let bytes = fs::read(path)?;
        let envelope = Envelope::from_bytes(&bytes)?;
        Ok(Self {
            path: path.to_path_buf(),
            envelope,
            data_key: None,
            vault: None,
            auto_backup_dir: None,
        })
    }

    pub fn unlock_with_password(&mut self, password: &str) -> Result<(), VaultError> {
        let (plaintext, data_key) = open_envelope(password.as_bytes(), &self.envelope)?;
        self.vault =
            Some(serde_json::from_slice(&plaintext).map_err(|e| VaultError::Malformed(e.to_string()))?);
        self.data_key = Some(data_key);
        Ok(())
    }

    pub fn unlock_with_slot(&mut self, slot: &str, wrapping_key: &[u8]) -> Result<(), VaultError> {
        let wrapped = self
            .envelope
            .header
            .wrapped_keys
            .get(slot)
            .cloned()
            .ok_or_else(|| VaultError::UnknownSlot(slot.to_string()))?;
        let wrapping = SymmetricKey::from_slice(wrapping_key)?;
        let data_key = unwrap_key(&wrapping, &wrapped.into())?;
        let plaintext = open_with_key(&data_key, &self.envelope)?;
        self.vault =
            Some(serde_json::from_slice(&plaintext).map_err(|e| VaultError::Malformed(e.to_string()))?);
        self.data_key = Some(data_key);
        Ok(())
    }

    pub fn add_key_slot(&mut self, slot: &str, wrapping_key: &[u8]) -> Result<(), VaultError> {
        let data_key = self.data_key.as_ref().ok_or(VaultError::Locked)?;
        let wrapping = SymmetricKey::from_slice(wrapping_key)?;
        let wrapped = wrap_key(&wrapping, data_key)?;
        self.envelope
            .header
            .wrapped_keys
            .insert(slot.to_string(), wrapped.into());
        self.persist()
    }

    pub fn remove_key_slot(&mut self, slot: &str) -> Result<(), VaultError> {
        if slot == "password" {
            return Err(VaultError::Malformed("password slot cannot be removed".into()));
        }
        self.envelope.header.wrapped_keys.remove(slot);
        self.persist()
    }

    pub fn has_key_slot(&self, slot: &str) -> bool {
        self.envelope.header.wrapped_keys.contains_key(slot)
    }

    pub fn is_unlocked(&self) -> bool {
        self.vault.is_some()
    }

    pub fn lock(&mut self) {
        self.vault = None;
        self.data_key = None;
    }

    pub fn vault(&self) -> Result<&Vault, VaultError> {
        self.vault.as_ref().ok_or(VaultError::Locked)
    }

    pub fn vault_mut(&mut self) -> Result<&mut Vault, VaultError> {
        self.vault.as_mut().ok_or(VaultError::Locked)
    }

    pub fn save(&mut self) -> Result<(), VaultError> {
        let vault = self.vault.as_ref().ok_or(VaultError::Locked)?;
        let data_key = self.data_key.as_ref().ok_or(VaultError::Locked)?;
        let plaintext = serde_json::to_vec(vault).map_err(|e| VaultError::Malformed(e.to_string()))?;
        reseal_with_key(data_key, &mut self.envelope, &plaintext)?;
        self.persist()
    }

    pub fn change_password(&mut self, current: &str, new: &str, params: KdfParams) -> Result<(), VaultError> {
        let (plaintext, _) = open_envelope(current.as_bytes(), &self.envelope)?;
        let (mut envelope, data_key) = seal_envelope(new.as_bytes(), &plaintext, PURPOSE, params)?;
        envelope.header.wrapped_keys.retain(|slot, _| slot == "password");
        self.envelope = envelope;
        self.data_key = Some(data_key);
        self.vault =
            Some(serde_json::from_slice(&plaintext).map_err(|e| VaultError::Malformed(e.to_string()))?);
        self.persist()
    }

    pub fn verify_password(&self, password: &str) -> bool {
        open_envelope(password.as_bytes(), &self.envelope).is_ok()
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Directory that receives an encrypted copy of the vault after every
    /// save. The copy is the sealed envelope itself, so it is protected by
    /// the same password and key slots as the vault.
    pub fn set_auto_backup_dir(&mut self, dir: Option<PathBuf>) {
        self.auto_backup_dir = dir;
    }

    pub fn auto_backup_dir(&self) -> Option<&Path> {
        self.auto_backup_dir.as_deref()
    }

    /// Runs the automatic startup pass: validates and repairs the vault
    /// structure and purges trash entries past the retention window.
    /// Persists only when something actually changed.
    pub fn startup_maintenance(&mut self, now: i64) -> Result<MaintenanceReport, VaultError> {
        let vault = self.vault.as_mut().ok_or(VaultError::Locked)?;
        let integrity = vault.check_integrity();
        let purged_from_trash = vault.purge_expired(now);
        let report = MaintenanceReport {
            integrity,
            purged_from_trash,
        };
        if report.integrity.repaired || report.purged_from_trash > 0 {
            self.save()?;
        }
        Ok(report)
    }

    fn persist(&self) -> Result<(), VaultError> {
        let bytes = self.envelope.to_bytes();
        let tmp = self.path.with_extension("tmp");
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&tmp, &bytes)?;
        fs::rename(&tmp, &self.path).or_else(|_| {
            fs::copy(&tmp, &self.path).map(|_| ())?;
            fs::remove_file(&tmp)
        })?;
        self.write_auto_backup(&bytes);
        Ok(())
    }

    /// Best effort: a failing backup target (unplugged drive, missing share)
    /// must never block saving the vault itself.
    fn write_auto_backup(&self, bytes: &[u8]) {
        let Some(dir) = self.auto_backup_dir.as_ref() else {
            return;
        };
        let _ = fs::create_dir_all(dir);
        let target = dir.join(AUTO_BACKUP_FILE);
        let tmp = target.with_extension("tmp");
        if fs::write(&tmp, bytes).is_ok() {
            let _ = fs::rename(&tmp, &target).or_else(|_| {
                fs::copy(&tmp, &target).map(|_| ())?;
                fs::remove_file(&tmp)
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use liauth_core::Account;
    use liauth_crypto::random_key;

    fn fast_params() -> KdfParams {
        KdfParams {
            memory_kib: 1024,
            iterations: 1,
            parallelism: 1,
        }
    }

    #[test]
    fn create_unlock_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vault.liauth");
        {
            let mut manager = VaultManager::create(&path, "pass", fast_params()).unwrap();
            let account = Account::new("GitHub".into(), "me".into(), b"secret".to_vec(), 0);
            manager.vault_mut().unwrap().accounts.push(account);
            manager.save().unwrap();
        }
        let mut manager = VaultManager::open(&path).unwrap();
        assert!(!manager.is_unlocked());
        manager.unlock_with_password("pass").unwrap();
        assert_eq!(manager.vault().unwrap().accounts.len(), 1);
        assert_eq!(manager.vault().unwrap().accounts[0].issuer, "GitHub");
    }

    #[test]
    fn wrong_password_fails() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vault.liauth");
        VaultManager::create(&path, "pass", fast_params()).unwrap();
        let mut manager = VaultManager::open(&path).unwrap();
        assert!(manager.unlock_with_password("nope").is_err());
    }

    #[test]
    fn platform_slot_unlocks() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vault.liauth");
        let platform_key = random_key();
        {
            let mut manager = VaultManager::create(&path, "pass", fast_params()).unwrap();
            manager.add_key_slot("platform", platform_key.bytes()).unwrap();
        }
        let mut manager = VaultManager::open(&path).unwrap();
        manager
            .unlock_with_slot("platform", platform_key.bytes())
            .unwrap();
        assert!(manager.is_unlocked());
    }

    #[test]
    fn auto_backup_written_on_save() {
        let dir = tempfile::tempdir().unwrap();
        let backup_dir = dir.path().join("backups");
        let path = dir.path().join("vault.liauth");
        let mut manager = VaultManager::create(&path, "pass", fast_params()).unwrap();
        manager.set_auto_backup_dir(Some(backup_dir.clone()));
        manager.vault_mut().unwrap().accounts.push(Account::new(
            "GitHub".into(),
            "me".into(),
            b"secret".to_vec(),
            0,
        ));
        manager.save().unwrap();

        let copy = backup_dir.join("LiAuth-auto-backup.liauth");
        assert!(copy.is_file());
        // The copy is a valid vault envelope openable with the same password.
        let mut restored = VaultManager::open(&copy).unwrap();
        restored.unlock_with_password("pass").unwrap();
        assert_eq!(restored.vault().unwrap().accounts.len(), 1);
    }

    #[test]
    fn startup_maintenance_purges_and_repairs() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vault.liauth");
        let mut manager = VaultManager::create(&path, "pass", fast_params()).unwrap();
        let mut stale = Account::new("Old".into(), "me".into(), b"secret".to_vec(), 0);
        stale.deleted_at = Some(0);
        let expired_at = crate::store::TRASH_RETENTION_SECONDS + 1;
        manager.vault_mut().unwrap().accounts.push(stale);
        let report = manager.startup_maintenance(expired_at).unwrap();
        assert_eq!(report.purged_from_trash, 1);
        assert!(manager.vault().unwrap().accounts.is_empty());
    }

    #[test]
    fn change_password_invalidates_slots() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vault.liauth");
        let platform_key = random_key();
        let mut manager = VaultManager::create(&path, "pass", fast_params()).unwrap();
        manager.add_key_slot("platform", platform_key.bytes()).unwrap();
        manager.change_password("pass", "next", fast_params()).unwrap();
        assert!(!manager.has_key_slot("platform"));

        let mut reopened = VaultManager::open(&path).unwrap();
        assert!(reopened.unlock_with_password("pass").is_err());
        reopened.unlock_with_password("next").unwrap();
    }
}
