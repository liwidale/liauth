use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("invalid base32 input")]
    InvalidBase32,
    #[error("invalid otpauth uri: {0}")]
    InvalidUri(String),
    #[error("invalid parameter: {0}")]
    InvalidParameter(String),
    #[error("cryptographic failure")]
    Crypto,
}
