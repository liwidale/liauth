use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Nonce};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::CryptoError;

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SymmetricKey([u8; 32]);

impl SymmetricKey {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn from_slice(bytes: &[u8]) -> Result<Self, CryptoError> {
        let array: [u8; 32] = bytes
            .try_into()
            .map_err(|_| CryptoError::Malformed("key must be 32 bytes".into()))?;
        Ok(Self(array))
    }

    pub fn bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WrappedKey {
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
}

pub fn random_key() -> SymmetricKey {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    SymmetricKey::new(bytes)
}

pub fn random_bytes(len: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; len];
    OsRng.fill_bytes(&mut bytes);
    bytes
}

pub fn seal(key: &SymmetricKey, plaintext: &[u8], aad: &[u8]) -> Result<(Vec<u8>, Vec<u8>), CryptoError> {
    let cipher = Aes256Gcm::new(key.bytes().into());
    let nonce_bytes = random_bytes(12);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, Payload { msg: plaintext, aad })
        .map_err(|_| CryptoError::Unauthenticated)?;
    Ok((nonce_bytes, ciphertext))
}

pub fn open(key: &SymmetricKey, nonce: &[u8], ciphertext: &[u8], aad: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if nonce.len() != 12 {
        return Err(CryptoError::Malformed("nonce must be 12 bytes".into()));
    }
    let cipher = Aes256Gcm::new(key.bytes().into());
    cipher
        .decrypt(Nonce::from_slice(nonce), Payload { msg: ciphertext, aad })
        .map_err(|_| CryptoError::Unauthenticated)
}

pub fn wrap_key(wrapping_key: &SymmetricKey, key: &SymmetricKey) -> Result<WrappedKey, CryptoError> {
    let (nonce, ciphertext) = seal(wrapping_key, key.bytes(), b"liauth.key")?;
    Ok(WrappedKey { nonce, ciphertext })
}

pub fn unwrap_key(wrapping_key: &SymmetricKey, wrapped: &WrappedKey) -> Result<SymmetricKey, CryptoError> {
    let bytes = open(wrapping_key, &wrapped.nonce, &wrapped.ciphertext, b"liauth.key")?;
    SymmetricKey::from_slice(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seal_open_roundtrip() {
        let key = random_key();
        let (nonce, ciphertext) = seal(&key, b"secret data", b"context").unwrap();
        assert_eq!(
            open(&key, &nonce, &ciphertext, b"context").unwrap(),
            b"secret data"
        );
    }

    #[test]
    fn tamper_detected() {
        let key = random_key();
        let (nonce, mut ciphertext) = seal(&key, b"secret data", b"context").unwrap();
        ciphertext[0] ^= 1;
        assert!(open(&key, &nonce, &ciphertext, b"context").is_err());
    }

    #[test]
    fn aad_mismatch_detected() {
        let key = random_key();
        let (nonce, ciphertext) = seal(&key, b"secret data", b"context").unwrap();
        assert!(open(&key, &nonce, &ciphertext, b"other").is_err());
    }

    #[test]
    fn wrap_unwrap_roundtrip() {
        let kek = random_key();
        let dek = random_key();
        let wrapped = wrap_key(&kek, &dek).unwrap();
        assert_eq!(unwrap_key(&kek, &wrapped).unwrap().bytes(), dek.bytes());
    }
}
