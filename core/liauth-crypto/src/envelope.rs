use std::collections::BTreeMap;

use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::kdf::{derive_key, KdfParams};
use crate::keys::{self, random_bytes, random_key, unwrap_key, wrap_key, SymmetricKey, WrappedKey};
use crate::CryptoError;

const FORMAT: &str = "liauth";
const VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvelopeHeader {
    pub format: String,
    pub version: u32,
    pub purpose: String,
    pub kdf: KdfParams,
    #[serde(with = "b64")]
    pub salt: Vec<u8>,
    pub wrapped_keys: BTreeMap<String, WrappedKeyEncoded>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WrappedKeyEncoded {
    #[serde(with = "b64")]
    pub nonce: Vec<u8>,
    #[serde(with = "b64")]
    pub ciphertext: Vec<u8>,
}

impl From<WrappedKey> for WrappedKeyEncoded {
    fn from(value: WrappedKey) -> Self {
        Self {
            nonce: value.nonce,
            ciphertext: value.ciphertext,
        }
    }
}

impl From<WrappedKeyEncoded> for WrappedKey {
    fn from(value: WrappedKeyEncoded) -> Self {
        Self {
            nonce: value.nonce,
            ciphertext: value.ciphertext,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Envelope {
    #[serde(flatten)]
    pub header: EnvelopeHeader,
    #[serde(with = "b64")]
    pub nonce: Vec<u8>,
    #[serde(with = "b64")]
    pub ciphertext: Vec<u8>,
}

impl Envelope {
    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec_pretty(self).expect("envelope serialization is infallible")
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CryptoError> {
        let envelope: Envelope =
            serde_json::from_slice(bytes).map_err(|e| CryptoError::Malformed(e.to_string()))?;
        if envelope.header.format != FORMAT {
            return Err(CryptoError::Malformed("not a liauth file".into()));
        }
        if envelope.header.version != VERSION {
            return Err(CryptoError::UnsupportedVersion(envelope.header.version));
        }
        Ok(envelope)
    }

    pub fn is_recognized(bytes: &[u8]) -> bool {
        serde_json::from_slice::<serde_json::Value>(bytes)
            .ok()
            .and_then(|v| v.get("format").and_then(|f| f.as_str()).map(|f| f == FORMAT))
            .unwrap_or(false)
    }
}

pub fn seal_envelope(
    password: &[u8],
    plaintext: &[u8],
    purpose: &str,
    params: KdfParams,
) -> Result<(Envelope, SymmetricKey), CryptoError> {
    let salt = random_bytes(16);
    let master_key = derive_key(password, &salt, &params)?;
    let data_key = random_key();
    let wrapped = wrap_key(&master_key, &data_key)?;
    let (nonce, ciphertext) = keys::seal(&data_key, plaintext, purpose.as_bytes())?;

    let mut wrapped_keys = BTreeMap::new();
    wrapped_keys.insert("password".to_string(), wrapped.into());

    let envelope = Envelope {
        header: EnvelopeHeader {
            format: FORMAT.to_string(),
            version: VERSION,
            purpose: purpose.to_string(),
            kdf: params,
            salt,
            wrapped_keys,
        },
        nonce,
        ciphertext,
    };
    Ok((envelope, data_key))
}

pub fn open_envelope(password: &[u8], envelope: &Envelope) -> Result<(Vec<u8>, SymmetricKey), CryptoError> {
    let master_key = derive_key(password, &envelope.header.salt, &envelope.header.kdf)?;
    let wrapped = envelope
        .header
        .wrapped_keys
        .get("password")
        .cloned()
        .ok_or_else(|| CryptoError::Malformed("missing password key slot".into()))?;
    let data_key = unwrap_key(&master_key, &wrapped.into())?;
    let plaintext = open_with_key(&data_key, envelope)?;
    Ok((plaintext, data_key))
}

pub fn open_with_key(data_key: &SymmetricKey, envelope: &Envelope) -> Result<Vec<u8>, CryptoError> {
    keys::open(
        data_key,
        &envelope.nonce,
        &envelope.ciphertext,
        envelope.header.purpose.as_bytes(),
    )
}

pub fn reseal_with_key(
    data_key: &SymmetricKey,
    envelope: &mut Envelope,
    plaintext: &[u8],
) -> Result<(), CryptoError> {
    let (nonce, ciphertext) = keys::seal(data_key, plaintext, envelope.header.purpose.as_bytes())?;
    envelope.nonce = nonce;
    envelope.ciphertext = ciphertext;
    Ok(())
}

mod b64 {
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&STANDARD.encode(bytes))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
        let encoded = String::deserialize(deserializer)?;
        STANDARD.decode(encoded).map_err(serde::de::Error::custom)
    }
}

pub fn encode_base64(bytes: &[u8]) -> String {
    B64.encode(bytes)
}

pub fn decode_base64(value: &str) -> Result<Vec<u8>, CryptoError> {
    B64.decode(value)
        .map_err(|e| CryptoError::Malformed(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fast_params() -> KdfParams {
        KdfParams {
            memory_kib: 1024,
            iterations: 1,
            parallelism: 1,
        }
    }

    #[test]
    fn seal_open_roundtrip() {
        let (envelope, _) = seal_envelope(b"correct horse", b"payload", "vault", fast_params()).unwrap();
        let bytes = envelope.to_bytes();
        let restored = Envelope::from_bytes(&bytes).unwrap();
        let (plaintext, _) = open_envelope(b"correct horse", &restored).unwrap();
        assert_eq!(plaintext, b"payload");
    }

    #[test]
    fn wrong_password_rejected() {
        let (envelope, _) = seal_envelope(b"correct horse", b"payload", "vault", fast_params()).unwrap();
        assert!(matches!(
            open_envelope(b"wrong", &envelope),
            Err(CryptoError::Unauthenticated)
        ));
    }

    #[test]
    fn purpose_binds_ciphertext() {
        let (mut envelope, _) = seal_envelope(b"correct horse", b"payload", "vault", fast_params()).unwrap();
        envelope.header.purpose = "backup".to_string();
        assert!(open_envelope(b"correct horse", &envelope).is_err());
    }

    #[test]
    fn extra_key_slot_unlocks_without_password() {
        let (mut envelope, data_key) =
            seal_envelope(b"correct horse", b"payload", "vault", fast_params()).unwrap();
        let platform_key = random_key();
        let wrapped = wrap_key(&platform_key, &data_key).unwrap();
        envelope
            .header
            .wrapped_keys
            .insert("platform".to_string(), wrapped.into());

        let restored = Envelope::from_bytes(&envelope.to_bytes()).unwrap();
        let slot: WrappedKey = restored
            .header
            .wrapped_keys
            .get("platform")
            .cloned()
            .unwrap()
            .into();
        let recovered = unwrap_key(&platform_key, &slot).unwrap();
        assert_eq!(open_with_key(&recovered, &restored).unwrap(), b"payload");
    }

    #[test]
    fn recognizes_format() {
        let (envelope, _) = seal_envelope(b"p", b"x", "backup", fast_params()).unwrap();
        assert!(Envelope::is_recognized(&envelope.to_bytes()));
        assert!(!Envelope::is_recognized(b"{\"format\":\"other\"}"));
        assert!(!Envelope::is_recognized(b"plain text"));
    }
}
