mod envelope;
mod kdf;
mod keys;

pub use envelope::{
    decode_base64, encode_base64, open_envelope, open_with_key, reseal_with_key, seal_envelope, Envelope,
    EnvelopeHeader, WrappedKeyEncoded,
};
pub use kdf::{derive_key, KdfParams};
pub use keys::{open, random_bytes, random_key, seal, unwrap_key, wrap_key, SymmetricKey, WrappedKey};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("wrong password or corrupted data")]
    Unauthenticated,
    #[error("unsupported format version {0}")]
    UnsupportedVersion(u32),
    #[error("malformed data: {0}")]
    Malformed(String),
    #[error("key derivation failed")]
    Kdf,
}
