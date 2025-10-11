pub mod config;
pub mod connector;
pub mod connectors;
pub mod deduplication;
pub mod importer;
pub mod mapper;
pub mod transaction;
pub mod validator;

pub use config::{ColumnMapping, ImportConfig, ImportFormat};
pub use connector::{ConnectorMetadata, ConnectorRegistry, ImportConnector, ParseResult};
pub use connectors::create_default_registry;
pub use deduplication::{
    DeduplicationConfig, DeduplicationEngine, DuplicateStrategy, MatchCriteria,
};
pub use importer::{ImportResult, ImportService};
pub use mapper::DataMapper;
pub use transaction::{ImportTransaction, TransactionState};
pub use validator::{BatchValidator, ValidationResult};

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
