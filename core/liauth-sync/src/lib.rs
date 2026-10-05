mod channel;
mod discovery;
mod receiver;
mod sender;
pub mod webdav;

pub use discovery::{discover, Peer, SERVICE_TYPE};
pub use receiver::{Receiver, ReceiverEvent};
pub use sender::send_payload;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SyncError {
    #[error("io failure: {0}")]
    Io(#[from] std::io::Error),
    #[error("pairing code mismatch")]
    PairingFailed,
    #[error("session closed unexpectedly")]
    Closed,
    #[error("message too large")]
    TooLarge,
    #[error("discovery failure: {0}")]
    Discovery(String),
    #[error("webdav failure: {0}")]
    WebDav(String),
    #[error("cryptographic failure")]
    Crypto,
}

pub fn generate_pairing_code() -> String {
    use rand::Rng;
    let value: u32 = rand::rngs::OsRng.gen_range(0..1_000_000);
    format!("{value:06}")
}
