pub mod config;
pub mod mapper;
pub mod validator;
pub mod importer;
pub mod transaction;

pub use config::{ImportConfig, ColumnMapping, ImportFormat};
pub use mapper::DataMapper;
pub use validator::{ValidationResult, BatchValidator};
pub use importer::{ImportService, ImportResult};
pub use transaction::{ImportTransaction, TransactionState};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ImportError {
    #[error("Validation failed: {0}")]
    ValidationError(String),

    #[error("Mapping error: {0}")]
    MappingError(String),

    #[error("Transaction error: {0}")]
    TransactionError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("CSV error: {0}")]
    CsvError(#[from] csv::Error),

    #[error("Other error: {0}")]
    Other(String),
}