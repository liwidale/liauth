use liauth_core::time::unix_now;
use liauth_core::{base32, uri, Account, TokenKind};
use serde::Deserialize;

use crate::ImportError;

#[derive(Deserialize)]
struct AuthyEntry {
    #[serde(default, alias = "label", alias = "original_name")]
    name: Option<String>,
    #[serde(default, alias = "decrypted_seed", alias = "secretSeed")]
    secret: Option<String>,
    #[serde(default)]
    issuer: Option<String>,
    #[serde(default)]
    digits: Option<u32>,
    #[serde(default)]
    period: Option<u32>,
    #[serde(default)]
    uri: Option<String>,
}

pub fn import_authy(text: &str) -> Result<Vec<Account>, ImportError> {
    let entries: Vec<AuthyEntry> =
        serde_json::from_str(text).map_err(|e| ImportError::Malformed(e.to_string()))?;
    let mut accounts = Vec::new();
    for entry in entries {
        if let Some(uri_value) = entry.uri.as_deref() {
            if let Ok(account) = uri::parse(uri_value) {
                accounts.push(account);
                continue;
            }
        }
        let Some(secret_str) = entry.secret.as_deref() else {
            continue;
        };
        let Ok(secret) = base32::decode(secret_str) else {
            continue;
        };
        let raw_name = entry.name.unwrap_or_default();
        let (issuer, name) = match entry.issuer {
            Some(issuer) if !issuer.is_empty() => (issuer, raw_name),
            _ => match raw_name.split_once(':') {
                Some((prefix, rest)) => (prefix.trim().to_string(), rest.trim().to_string()),
                None => (raw_name, String::new()),
            },
        };
        let mut account = Account::new(issuer, name, secret, unix_now());
        account.digits = entry.digits.filter(|d| (4..=10).contains(d)).unwrap_or(6);
        account.kind = TokenKind::Totp {
            period: entry.period.filter(|p| *p >= 5).unwrap_or(30),
        };
        accounts.push(account);
    }
    if accounts.is_empty() {
        return Err(ImportError::Empty);
    }
    Ok(accounts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imports_exported_entries() {
        let json = r#"[
            {"name": "GitHub: liwidale", "secret": "MZXW6YTBOI", "digits": 7, "period": 10},
            {"uri": "otpauth://totp/GitLab:me?secret=MZXW6YTBOI&issuer=GitLab"}
        ]"#;
        let accounts = import_authy(json).unwrap();
        assert_eq!(accounts.len(), 2);
        assert_eq!(accounts[0].issuer, "GitHub");
        assert_eq!(accounts[0].name, "liwidale");
        assert_eq!(accounts[0].digits, 7);
        assert_eq!(accounts[0].kind, TokenKind::Totp { period: 10 });
        assert_eq!(accounts[1].issuer, "GitLab");
    }

    #[test]
    fn skips_invalid_entries() {
        let json = r#"[{"name": "broken"}, {"name": "ok", "secret": "MZXW6YTBOI"}]"#;
        let accounts = import_authy(json).unwrap();
        assert_eq!(accounts.len(), 1);
    }
}
