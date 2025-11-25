use thiserror::Error;

pub type CryptoResult<T> = Result<T, CryptoError>;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Encryption failed: {0}")]
    EncryptionError(String),

    #[error("Decryption failed: {0}")]
    DecryptionError(String),

    #[error("Key derivation failed: {0}")]
    KeyDerivationError(String),

    #[error("Invalid key version: {0}")]
    InvalidKeyVersion(i32),

    #[error("Key not found: {0}")]
    KeyNotFound(i32),

    #[error("Invalid key format: {0}")]
    InvalidKeyFormat(String),

    #[error("Invalid encrypted data format: {0}")]
    InvalidDataFormat(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Key rotation failed: {0}")]
    RotationError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Base64 decode error: {0}")]
    Base64Error(#[from] base64::DecodeError),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("UTF-8 error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}

#[cfg(feature = "async")]
impl From<sqlx::Error> for CryptoError {
    fn from(err: sqlx::Error) -> Self {
        CryptoError::DatabaseError(err.to_string())
    }
}
