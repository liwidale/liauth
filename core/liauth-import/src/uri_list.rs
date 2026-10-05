use liauth_core::{uri, Account};

use crate::google::import_google_migration;
use crate::ImportError;

pub fn import_uri_list(text: &str) -> Result<Vec<Account>, ImportError> {
    let mut accounts = Vec::new();
    for line in text.lines().map(str::trim).filter(|l| !l.is_empty()) {
        if line.starts_with("otpauth-migration://") {
            accounts.extend(import_google_migration(line)?);
        } else if line.starts_with("otpauth://") {
            accounts.push(uri::parse(line).map_err(|e| ImportError::Malformed(e.to_string()))?);
        }
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
    fn imports_multiple_lines() {
        let text = "otpauth://totp/GitHub:me?secret=MZXW6YTBOI&issuer=GitHub\n\notpauth://totp/GitLab:me?secret=MZXW6YTBOI&issuer=GitLab\n";
        let accounts = import_uri_list(text).unwrap();
        assert_eq!(accounts.len(), 2);
    }

    #[test]
    fn rejects_empty_input() {
        assert!(import_uri_list("\n\n").is_err());
    }
}
