use thiserror::Error;

#[derive(Debug, Error)]
pub enum AttachmentError {
    #[error("File too large: {0} bytes (max: {1} bytes)")]
    FileTooLarge(i64, i64),

    #[error("Invalid content type: {0}")]
    InvalidContentType(String),

    #[error("Virus detected: {0}")]
    VirusDetected(String),

    #[error("Scan failed: {0}")]
    ScanFailed(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Attachment not found: {0}")]
    NotFound(uuid::Uuid),

    #[error("Invalid checksum")]
    InvalidChecksum,

    #[error("Encryption error: {0}")]
    EncryptionError(String),

    #[error("Other error: {0}")]
    Other(String),
}