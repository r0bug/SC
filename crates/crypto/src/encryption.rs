use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::RngCore;
use serde::{Deserialize, Serialize};

use crate::errors::{CryptoError, CryptoResult};

const NONCE_SIZE: usize = 12; // 96 bits for AES-GCM

/// Encrypted data with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    /// Base64-encoded ciphertext
    pub ciphertext: String,
    /// Base64-encoded nonce
    pub nonce: String,
    /// Key version used for encryption
    pub key_version: i32,
}

/// Encrypt data using AES-256-GCM
pub fn encrypt_data(plaintext: &[u8], key: &[u8; 32], key_version: i32) -> CryptoResult<EncryptedData> {
    // Create cipher
    let cipher = Aes256Gcm::new(key.into());

    // Generate random nonce
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| CryptoError::EncryptionError(e.to_string()))?;

    Ok(EncryptedData {
        ciphertext: BASE64.encode(&ciphertext),
        nonce: BASE64.encode(nonce_bytes),
        key_version,
    })
}

/// Decrypt data using AES-256-GCM
pub fn decrypt_data(encrypted: &EncryptedData, key: &[u8; 32]) -> CryptoResult<Vec<u8>> {
    // Decode ciphertext and nonce
    let ciphertext = BASE64
        .decode(&encrypted.ciphertext)
        .map_err(|e| CryptoError::InvalidDataFormat(format!("Invalid ciphertext: {}", e)))?;

    let nonce_bytes = BASE64
        .decode(&encrypted.nonce)
        .map_err(|e| CryptoError::InvalidDataFormat(format!("Invalid nonce: {}", e)))?;

    if nonce_bytes.len() != NONCE_SIZE {
        return Err(CryptoError::InvalidDataFormat(format!(
            "Invalid nonce size: expected {}, got {}",
            NONCE_SIZE,
            nonce_bytes.len()
        )));
    }

    let nonce = Nonce::from_slice(&nonce_bytes);

    // Create cipher and decrypt
    let cipher = Aes256Gcm::new(key.into());
    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| CryptoError::DecryptionError(e.to_string()))?;

    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = [0u8; 32];
        let plaintext = b"Hello, World!";

        let encrypted = encrypt_data(plaintext, &key, 1).unwrap();
        let decrypted = decrypt_data(&encrypted, &key).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_different_nonces() {
        let key = [0u8; 32];
        let plaintext = b"Hello, World!";

        let encrypted1 = encrypt_data(plaintext, &key, 1).unwrap();
        let encrypted2 = encrypt_data(plaintext, &key, 1).unwrap();

        // Same plaintext with same key should produce different ciphertexts (different nonces)
        assert_ne!(encrypted1.ciphertext, encrypted2.ciphertext);
        assert_ne!(encrypted1.nonce, encrypted2.nonce);

        // Both should decrypt correctly
        let decrypted1 = decrypt_data(&encrypted1, &key).unwrap();
        let decrypted2 = decrypt_data(&encrypted2, &key).unwrap();

        assert_eq!(plaintext, decrypted1.as_slice());
        assert_eq!(plaintext, decrypted2.as_slice());
    }

    #[test]
    fn test_wrong_key() {
        let key1 = [0u8; 32];
        let key2 = [1u8; 32];
        let plaintext = b"Hello, World!";

        let encrypted = encrypt_data(plaintext, &key1, 1).unwrap();
        let result = decrypt_data(&encrypted, &key2);

        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_ciphertext() {
        let key = [0u8; 32];
        let encrypted = EncryptedData {
            ciphertext: "not-valid-base64!@#".to_string(),
            nonce: BASE64.encode([0u8; NONCE_SIZE]),
            key_version: 1,
        };

        let result = decrypt_data(&encrypted, &key);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_nonce_size() {
        let key = [0u8; 32];
        let encrypted = EncryptedData {
            ciphertext: BASE64.encode(b"some ciphertext"),
            nonce: BASE64.encode([0u8; 8]), // Wrong size
            key_version: 1,
        };

        let result = decrypt_data(&encrypted, &key);
        assert!(result.is_err());
    }
}
