use std::io::{Read, Write};
use std::net::TcpStream;

use hkdf::Hkdf;
use liauth_crypto::{open, seal, SymmetricKey};
use sha2::Sha256;
use spake2::{Ed25519Group, Identity, Password, Spake2};

use crate::SyncError;

const MAX_FRAME: usize = 16 * 1024 * 1024;
const SENDER_ID: &[u8] = b"liauth-sync-sender";
const RECEIVER_ID: &[u8] = b"liauth-sync-receiver";
const AAD: &[u8] = b"liauth.sync.v1";

pub struct SecureChannel {
    stream: TcpStream,
    key: SymmetricKey,
}

impl SecureChannel {
    pub fn establish_sender(mut stream: TcpStream, code: &str) -> Result<Self, SyncError> {
        let (state, outbound) = Spake2::<Ed25519Group>::start_a(
            &Password::new(code.as_bytes()),
            &Identity::new(SENDER_ID),
            &Identity::new(RECEIVER_ID),
        );
        write_frame(&mut stream, &outbound)?;
        let inbound = read_frame(&mut stream)?;
        let shared = state.finish(&inbound).map_err(|_| SyncError::PairingFailed)?;
        Ok(Self {
            stream,
            key: derive_session_key(&shared)?,
        })
    }

    pub fn establish_receiver(mut stream: TcpStream, code: &str) -> Result<Self, SyncError> {
        let (state, outbound) = Spake2::<Ed25519Group>::start_b(
            &Password::new(code.as_bytes()),
            &Identity::new(SENDER_ID),
            &Identity::new(RECEIVER_ID),
        );
        let inbound = read_frame(&mut stream)?;
        write_frame(&mut stream, &outbound)?;
        let shared = state.finish(&inbound).map_err(|_| SyncError::PairingFailed)?;
        Ok(Self {
            stream,
            key: derive_session_key(&shared)?,
        })
    }

    pub fn send(&mut self, payload: &[u8]) -> Result<(), SyncError> {
        let (nonce, ciphertext) = seal(&self.key, payload, AAD).map_err(|_| SyncError::Crypto)?;
        let mut frame = nonce;
        frame.extend_from_slice(&ciphertext);
        write_frame(&mut self.stream, &frame)
    }

    pub fn receive(&mut self) -> Result<Vec<u8>, SyncError> {
        let frame = read_frame(&mut self.stream)?;
        if frame.len() < 12 {
            return Err(SyncError::Closed);
        }
        open(&self.key, &frame[..12], &frame[12..], AAD).map_err(|_| SyncError::PairingFailed)
    }
}

fn derive_session_key(shared: &[u8]) -> Result<SymmetricKey, SyncError> {
    let hkdf = Hkdf::<Sha256>::new(None, shared);
    let mut key = [0u8; 32];
    hkdf.expand(AAD, &mut key).map_err(|_| SyncError::Crypto)?;
    Ok(SymmetricKey::new(key))
}

fn write_frame(stream: &mut TcpStream, data: &[u8]) -> Result<(), SyncError> {
    if data.len() > MAX_FRAME {
        return Err(SyncError::TooLarge);
    }
    stream.write_all(&(data.len() as u32).to_be_bytes())?;
    stream.write_all(data)?;
    stream.flush()?;
    Ok(())
}

fn read_frame(stream: &mut TcpStream) -> Result<Vec<u8>, SyncError> {
    let mut len_bytes = [0u8; 4];
    stream.read_exact(&mut len_bytes)?;
    let len = u32::from_be_bytes(len_bytes) as usize;
    if len > MAX_FRAME {
        return Err(SyncError::TooLarge);
    }
    let mut buffer = vec![0u8; len];
    stream.read_exact(&mut buffer)?;
    Ok(buffer)
}
