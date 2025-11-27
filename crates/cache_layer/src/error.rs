//! Cache error types

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    #[error("Key not found: {0}")]
    NotFound(String),

    #[error("Operation timeout")]
    Timeout,

    #[error("Cache backend unavailable")]
    Unavailable,

    #[error("Invalid configuration: {0}")]
    Configuration(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<serde_json::Error> for CacheError {
    fn from(err: serde_json::Error) -> Self {
        CacheError::Serialization(err.to_string())
    }
}

#[cfg(feature = "redis")]
impl From<redis::RedisError> for CacheError {
    fn from(err: redis::RedisError) -> Self {
        CacheError::Connection(err.to_string())
    }
}
