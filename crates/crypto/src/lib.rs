//! Cryptographic utilities for SagensContact
//!
//! This crate provides encryption, key management, and key rotation capabilities.
//!
//! Features:
//! - `async`: Enables async key management and rotation (requires tokio and sqlx)

mod errors;
mod encryption;
mod key_management;

#[cfg(feature = "async")]
mod rotation;

#[cfg(test)]
mod tests;

pub use errors::{CryptoError, CryptoResult};
pub use encryption::{decrypt_data, encrypt_data, EncryptedData};
pub use key_management::{derive_key_from_password, generate_salt, KeyManager, KeyVersion};

#[cfg(feature = "async")]
pub use rotation::rotate_key;
