use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use liauth_core::time::unix_now;
use liauth_core::{base32, Account, Algorithm, TokenKind};
use scrypt::{scrypt, Params as ScryptParams};
use serde::Deserialize;

use crate::ImportError;

#[derive(Deserialize)]
struct AegisFile {
    header: AegisHeader,
    db: serde_json::Value,
}

#[derive(Deserialize)]
struct AegisHeader {
    slots: Option<Vec<AegisSlot>>,
    params: Option<AegisAeadParams>,
}

#[derive(Deserialize)]
struct AegisSlot {
    #[serde(rename = "type")]
    slot_type: u32,
    key: String,
    key_params: AegisAeadParams,
    n: Option<u32>,
    r: Option<u32>,
    p: Option<u32>,
    salt: Option<String>,
}

#[derive(Deserialize)]
struct AegisAeadParams {
    nonce: String,
    tag: String,
}

#[derive(Deserialize)]
struct AegisDb {
    entries: Vec<AegisEntry>,
}

#[derive(Deserialize)]
struct AegisEntry {
    #[serde(rename = "type")]
    entry_type: String,
    name: String,
    #[serde(default)]
    issuer: String,
    info: AegisInfo,
}

#[derive(Deserialize)]
struct AegisInfo {
    secret: String,
    #[serde(default)]
    algo: Option<String>,
    #[serde(default)]
    digits: Option<u32>,
    #[serde(default)]
    period: Option<u32>,
    #[serde(default)]
    counter: Option<u64>,
}

pub fn import_aegis(text: &str, password: Option<&str>) -> Result<Vec<Account>, ImportError> {
    let file: AegisFile = serde_json::from_str(text).map_err(|e| ImportError::Malformed(e.to_string()))?;

    let db: AegisDb = match &file.db {
        serde_json::Value::Object(_) => {
            serde_json::from_value(file.db.clone()).map_err(|e| ImportError::Malformed(e.to_string()))?
        }
        serde_json::Value::String(encrypted) => {
            let password = password.ok_or(ImportError::PasswordRequired)?;
            let params = file
                .header
                .params
                .as_ref()
                .ok_or_else(|| ImportError::Malformed("missing aead params".into()))?;
            let slots = file
                .header
                .slots
                .as_ref()
                .ok_or_else(|| ImportError::Malformed("missing key slots".into()))?;
            let master_key = unlock_master_key(slots, password)?;
            let ciphertext = B64
                .decode(encrypted.as_bytes())
                .map_err(|_| ImportError::Malformed("invalid db base64".into()))?;
            let plaintext = aes_open(&master_key, params, &ciphertext)?;
            serde_json::from_slice(&plaintext).map_err(|e| ImportError::Malformed(e.to_string()))?
        }
        _ => return Err(ImportError::Malformed("unexpected db field".into())),
    };

    let mut accounts = Vec::new();
    for entry in db.entries {
        let secret = base32::decode(&entry.info.secret)
            .map_err(|_| ImportError::Malformed("invalid entry secret".into()))?;
        let mut account = Account::new(entry.issuer, entry.name, secret, unix_now());
        account.algorithm = entry
            .info
            .algo
            .as_deref()
            .and_then(Algorithm::parse)
            .unwrap_or_default();
        account.digits = entry.info.digits.filter(|d| (4..=10).contains(d)).unwrap_or(6);
        account.kind = match entry.entry_type.as_str() {
            "hotp" => TokenKind::Hotp {
                counter: entry.info.counter.unwrap_or(0),
            },
            "steam" => TokenKind::Steam,
            _ => TokenKind::Totp {
                period: entry.info.period.filter(|p| *p >= 5).unwrap_or(30),
            },
        };
        accounts.push(account);
    }
    if accounts.is_empty() {
        return Err(ImportError::Empty);
    }
    Ok(accounts)
}

fn unlock_master_key(slots: &[AegisSlot], password: &str) -> Result<[u8; 32], ImportError> {
    for slot in slots.iter().filter(|s| s.slot_type == 1) {
        let salt = hex_decode(slot.salt.as_deref().unwrap_or_default())?;
        let n = slot.n.unwrap_or(32768);
        let log_n = n.ilog2() as u8;
        let params = ScryptParams::new(log_n, slot.r.unwrap_or(8), slot.p.unwrap_or(1), 32)
            .map_err(|_| ImportError::Malformed("invalid scrypt params".into()))?;
        let mut derived = [0u8; 32];
        scrypt(password.as_bytes(), &salt, &params, &mut derived)
            .map_err(|_| ImportError::Malformed("scrypt failure".into()))?;

        let encrypted_key = hex_decode(&slot.key)?;
        if let Ok(master) = aes_open_raw(&derived, &slot.key_params, &encrypted_key) {
            let master: [u8; 32] = master
                .try_into()
                .map_err(|_| ImportError::Malformed("bad master key size".into()))?;
            return Ok(master);
        }
    }
    Err(ImportError::WrongPassword)
}

fn aes_open(key: &[u8; 32], params: &AegisAeadParams, ciphertext: &[u8]) -> Result<Vec<u8>, ImportError> {
    aes_open_raw(key, params, ciphertext).map_err(|_| ImportError::WrongPassword)
}

fn aes_open_raw(key: &[u8; 32], params: &AegisAeadParams, ciphertext: &[u8]) -> Result<Vec<u8>, ImportError> {
    let nonce = hex_decode(&params.nonce)?;
    let tag = hex_decode(&params.tag)?;
    let mut combined = ciphertext.to_vec();
    combined.extend_from_slice(&tag);
    let cipher = Aes256Gcm::new(key.into());
    cipher
        .decrypt(
            Nonce::from_slice(&nonce),
            Payload {
                msg: &combined,
                aad: &[],
            },
        )
        .map_err(|_| ImportError::WrongPassword)
}

fn hex_decode(input: &str) -> Result<Vec<u8>, ImportError> {
    if input.len() % 2 != 0 {
        return Err(ImportError::Malformed("odd hex length".into()));
    }
    (0..input.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&input[i..i + 2], 16).map_err(|_| ImportError::Malformed("invalid hex".into()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imports_plain_export() {
        let json = r#"{
            "version": 1,
            "header": {"slots": null, "params": null},
            "db": {
                "version": 2,
                "entries": [
                    {
                        "type": "totp",
                        "uuid": "b3a7-1",
                        "name": "liwidale",
                        "issuer": "GitHub",
                        "info": {"secret": "MZXW6YTBOI", "algo": "SHA256", "digits": 8, "period": 60}
                    },
                    {
                        "type": "hotp",
                        "uuid": "b3a7-2",
                        "name": "vendor",
                        "issuer": "Vendor",
                        "info": {"secret": "MZXW6YTBOI", "counter": 4}
                    }
                ]
            }
        }"#;
        let accounts = import_aegis(json, None).unwrap();
        assert_eq!(accounts.len(), 2);
        assert_eq!(accounts[0].issuer, "GitHub");
        assert_eq!(accounts[0].digits, 8);
        assert_eq!(accounts[0].algorithm, Algorithm::Sha256);
        assert_eq!(accounts[1].kind, TokenKind::Hotp { counter: 4 });
    }

    #[test]
    fn encrypted_requires_password() {
        let json = r#"{
            "version": 1,
            "header": {
                "slots": [{"type":1,"uuid":"x","key":"00","key_params":{"nonce":"00","tag":"00"},"n":16384,"r":8,"p":1,"salt":"00"}],
                "params": {"nonce": "00", "tag": "00"}
            },
            "db": "AAAA"
        }"#;
        assert!(matches!(
            import_aegis(json, None),
            Err(ImportError::PasswordRequired)
        ));
    }

    #[test]
    fn hex_roundtrip() {
        assert_eq!(hex_decode("00ff10").unwrap(), vec![0x00, 0xff, 0x10]);
        assert!(hex_decode("0").is_err());
        assert!(hex_decode("zz").is_err());
    }
}
