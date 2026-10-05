//! Persistent unlock-attempt tracking. Failed attempts and their timestamp
//! live in a small sidecar file next to the vault, so the progressive delay
//! survives an app restart. The file contains no secret material.

use std::fs;
use std::path::{Path, PathBuf};

use liauth_core::lockout::delay_seconds;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
struct LockoutState {
    failures: u32,
    last_failure_at: i64,
}

#[derive(Debug)]
pub struct Lockout {
    path: PathBuf,
    state: LockoutState,
}

impl Lockout {
    /// Loads the lockout state stored next to `vault_path`.
    pub fn for_vault(vault_path: &Path) -> Self {
        let path = vault_path.with_extension("lockout");
        let state = fs::read(&path)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default();
        Self { path, state }
    }

    /// Seconds left before another unlock attempt is allowed; 0 when free.
    pub fn remaining_delay(&self, now: i64) -> u64 {
        let delay = delay_seconds(self.state.failures) as i64;
        let elapsed = now - self.state.last_failure_at;
        if elapsed >= delay {
            0
        } else {
            (delay - elapsed.max(0)) as u64
        }
    }

    pub fn failures(&self) -> u32 {
        self.state.failures
    }

    pub fn record_failure(&mut self, now: i64) {
        self.state.failures = self.state.failures.saturating_add(1);
        self.state.last_failure_at = now;
        self.persist();
    }

    pub fn record_success(&mut self) {
        self.state = LockoutState::default();
        let _ = fs::remove_file(&self.path);
    }

    fn persist(&self) {
        if let Ok(bytes) = serde_json::to_vec(&self.state) {
            let _ = fs::write(&self.path, bytes);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delay_kicks_in_and_persists() {
        let dir = tempfile::tempdir().unwrap();
        let vault_path = dir.path().join("vault.liauth");

        let mut lockout = Lockout::for_vault(&vault_path);
        assert_eq!(lockout.remaining_delay(1000), 0);
        lockout.record_failure(1000);
        lockout.record_failure(1001);
        lockout.record_failure(1002);
        assert!(lockout.remaining_delay(1002) > 0);

        // Reloaded from disk, the counter is still there.
        let reloaded = Lockout::for_vault(&vault_path);
        assert_eq!(reloaded.failures(), 3);
        assert!(reloaded.remaining_delay(1002) > 0);
        assert_eq!(reloaded.remaining_delay(1002 + 3600), 0);
    }

    #[test]
    fn success_clears_state() {
        let dir = tempfile::tempdir().unwrap();
        let vault_path = dir.path().join("vault.liauth");
        let mut lockout = Lockout::for_vault(&vault_path);
        for i in 0..5 {
            lockout.record_failure(i);
        }
        lockout.record_success();
        assert_eq!(lockout.failures(), 0);
        assert_eq!(Lockout::for_vault(&vault_path).failures(), 0);
    }
}
