use argon2::{Algorithm, Argon2, Params, Version};
use serde::{Deserialize, Serialize};

use crate::keys::SymmetricKey;
use crate::CryptoError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KdfParams {
    pub memory_kib: u32,
    pub iterations: u32,
    pub parallelism: u32,
}

impl Default for KdfParams {
    fn default() -> Self {
        Self {
            memory_kib: 65536,
            iterations: 3,
            parallelism: 2,
        }
    }
}

impl KdfParams {
    pub fn mobile() -> Self {
        Self {
            memory_kib: 19456,
            iterations: 2,
            parallelism: 1,
        }
    }
}

pub fn derive_key(password: &[u8], salt: &[u8], params: &KdfParams) -> Result<SymmetricKey, CryptoError> {
    let argon_params = Params::new(params.memory_kib, params.iterations, params.parallelism, Some(32))
        .map_err(|_| CryptoError::Kdf)?;
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, argon_params);
    let mut key = [0u8; 32];
    argon
        .hash_password_into(password, salt, &mut key)
        .map_err(|_| CryptoError::Kdf)?;
    Ok(SymmetricKey::new(key))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic() {
        let params = KdfParams {
            memory_kib: 1024,
            iterations: 1,
            parallelism: 1,
        };
        let a = derive_key(b"password", b"0123456789abcdef", &params).unwrap();
        let b = derive_key(b"password", b"0123456789abcdef", &params).unwrap();
        assert_eq!(a.bytes(), b.bytes());
    }

    #[test]
    fn salt_changes_output() {
        let params = KdfParams {
            memory_kib: 1024,
            iterations: 1,
            parallelism: 1,
        };
        let a = derive_key(b"password", b"0123456789abcdef", &params).unwrap();
        let b = derive_key(b"password", b"fedcba9876543210", &params).unwrap();
        assert_ne!(a.bytes(), b.bytes());
    }
}
