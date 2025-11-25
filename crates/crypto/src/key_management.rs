use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

use crate::errors::{CryptoError, CryptoResult};

#[cfg(feature = "async")]
use sqlx::SqlitePool;

#[cfg(feature = "async")]
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};

#[cfg(feature = "async")]
use tracing::debug;

#[cfg(feature = "async")]
use uuid::Uuid;

/// Represents a cryptographic key with version and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyVersion {
    pub version: i32,
    pub key_hash: String,
    pub created_at: DateTime<Utc>,
    pub is_active: bool,
}

/// Manages encryption keys with versioning support
pub struct KeyManager {
    keys: HashMap<i32, [u8; 32]>,
    current_version: i32,
    #[cfg(feature = "async")]
    pool: Option<SqlitePool>,
}

impl KeyManager {
    /// Create a new KeyManager with an initial key
    pub fn new() -> CryptoResult<Self> {
        let mut manager = Self {
            keys: HashMap::new(),
            current_version: 0,
            #[cfg(feature = "async")]
            pool: None,
        };

        // Generate initial key
        manager.generate_new_key()?;

        Ok(manager)
    }

    /// Create a KeyManager from an existing master password
    pub fn from_password(password: &str, salt: &str) -> CryptoResult<Self> {
        let key = derive_key_from_password(password, salt)?;

        let mut manager = Self {
            keys: HashMap::new(),
            current_version: 1,
            #[cfg(feature = "async")]
            pool: None,
        };

        manager.keys.insert(1, key);

        Ok(manager)
    }

    /// Create a KeyManager with database persistence (async feature required)
    #[cfg(feature = "async")]
    pub async fn with_database(pool: SqlitePool) -> CryptoResult<Self> {
        let mut manager = Self {
            keys: HashMap::new(),
            current_version: 0,
            pool: Some(pool),
        };

        // Load existing keys from database
        manager.load_keys_from_db().await?;

        // If no keys exist, generate initial key
        if manager.keys.is_empty() {
            manager.generate_new_key()?;
            manager.save_current_key_to_db().await?;
        }

        Ok(manager)
    }

    /// Generate a new encryption key
    fn generate_new_key(&mut self) -> CryptoResult<()> {
        use rand::RngCore;

        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);

        self.current_version += 1;
        self.keys.insert(self.current_version, key);

        info!(
            "Generated new encryption key version {}",
            self.current_version
        );

        Ok(())
    }

    /// Get the current active key
    pub fn get_current_key(&self) -> CryptoResult<(&[u8; 32], i32)> {
        self.keys
            .get(&self.current_version)
            .map(|k| (k, self.current_version))
            .ok_or(CryptoError::KeyNotFound(self.current_version))
    }

    /// Get a key by version (for decryption of old data)
    pub fn get_key(&self, version: i32) -> CryptoResult<&[u8; 32]> {
        self.keys
            .get(&version)
            .ok_or(CryptoError::KeyNotFound(version))
    }

    /// Get the current key version
    pub fn current_version(&self) -> i32 {
        self.current_version
    }

    /// Rotate to a new key version
    pub fn rotate_key(&mut self) -> CryptoResult<i32> {
        self.generate_new_key()?;
        Ok(self.current_version)
    }

    /// Add an existing key (used during key loading)
    pub fn add_key(&mut self, version: i32, key: [u8; 32]) {
        self.keys.insert(version, key);
        if version > self.current_version {
            self.current_version = version;
        }
    }

    /// Load keys from database (async feature required)
    #[cfg(feature = "async")]
    async fn load_keys_from_db(&mut self) -> CryptoResult<()> {
        let pool = self.pool.as_ref().ok_or_else(|| {
            CryptoError::DatabaseError("No database pool configured".to_string())
        })?;

        let rows = sqlx::query_as::<_, (i32, String, bool)>(
            "SELECT version, key_material, is_active FROM encryption_keys ORDER BY version ASC"
        )
        .fetch_all(pool)
        .await?;

        for (version, key_material, is_active) in rows {
            let key_bytes = BASE64.decode(&key_material).map_err(|e| {
                CryptoError::InvalidKeyFormat(format!("Failed to decode key: {}", e))
            })?;

            if key_bytes.len() != 32 {
                return Err(CryptoError::InvalidKeyFormat(format!(
                    "Invalid key size: expected 32, got {}",
                    key_bytes.len()
                )));
            }

            let mut key = [0u8; 32];
            key.copy_from_slice(&key_bytes);

            self.add_key(version, key);

            if is_active {
                self.current_version = version;
            }

            debug!("Loaded encryption key version {}", version);
        }

        info!("Loaded {} encryption keys from database", self.keys.len());

        Ok(())
    }

    /// Save the current key to database (async feature required)
    #[cfg(feature = "async")]
    async fn save_current_key_to_db(&self) -> CryptoResult<()> {
        let pool = self.pool.as_ref().ok_or_else(|| {
            CryptoError::DatabaseError("No database pool configured".to_string())
        })?;

        let (key, version) = self.get_current_key()?;
        let key_material = BASE64.encode(key);
        let id = Uuid::new_v4().to_string();

        sqlx::query(
            "INSERT INTO encryption_keys (id, version, key_material, is_active) VALUES (?, ?, ?, 1)"
        )
        .bind(&id)
        .bind(version)
        .bind(&key_material)
        .execute(pool)
        .await?;

        debug!("Saved encryption key version {} to database", version);

        Ok(())
    }

    /// Mark a key version as active in database (async feature required)
    #[cfg(feature = "async")]
    pub async fn set_active_version(&mut self, version: i32) -> CryptoResult<()> {
        let pool = self.pool.as_ref().ok_or_else(|| {
            CryptoError::DatabaseError("No database pool configured".to_string())
        })?;

        // Verify key exists
        if !self.keys.contains_key(&version) {
            return Err(CryptoError::KeyNotFound(version));
        }

        // Deactivate all keys
        sqlx::query("UPDATE encryption_keys SET is_active = 0")
            .execute(pool)
            .await?;

        // Activate specified version
        sqlx::query("UPDATE encryption_keys SET is_active = 1 WHERE version = ?")
            .bind(version)
            .execute(pool)
            .await?;

        self.current_version = version;

        info!("Set encryption key version {} as active", version);

        Ok(())
    }
}

impl Default for KeyManager {
    fn default() -> Self {
        Self::new().expect("Failed to create default KeyManager")
    }
}

/// Derive a 256-bit key from a password using Argon2
pub fn derive_key_from_password(password: &str, salt: &str) -> CryptoResult<[u8; 32]> {
    let argon2 = Argon2::default();

    let salt_string = SaltString::from_b64(salt)
        .map_err(|e| CryptoError::KeyDerivationError(format!("Invalid salt: {}", e)))?;

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_string)
        .map_err(|e| CryptoError::KeyDerivationError(e.to_string()))?;

    let hash = password_hash.hash.ok_or_else(|| {
        CryptoError::KeyDerivationError("Failed to extract hash".to_string())
    })?;

    let hash_bytes = hash.as_bytes();
    if hash_bytes.len() < 32 {
        return Err(CryptoError::KeyDerivationError(
            "Hash too short".to_string(),
        ));
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(&hash_bytes[..32]);

    Ok(key)
}

/// Generate a random salt for password-based key derivation
pub fn generate_salt() -> String {
    SaltString::generate(&mut OsRng).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_manager_creation() {
        let manager = KeyManager::new().unwrap();
        assert_eq!(manager.current_version(), 1);
        assert!(manager.get_current_key().is_ok());
    }

    #[test]
    fn test_key_rotation() {
        let mut manager = KeyManager::new().unwrap();
        let initial_version = manager.current_version();

        let new_version = manager.rotate_key().unwrap();
        assert_eq!(new_version, initial_version + 1);
        assert_eq!(manager.current_version(), new_version);
    }

    #[test]
    fn test_get_old_key() {
        let mut manager = KeyManager::new().unwrap();
        let (old_key, old_version) = manager.get_current_key().unwrap();
        let old_key = *old_key;

        manager.rotate_key().unwrap();

        // Should still be able to get old key
        let retrieved_old_key = manager.get_key(old_version).unwrap();
        assert_eq!(&old_key, retrieved_old_key);
    }

    #[test]
    fn test_password_derivation() {
        let salt = generate_salt();
        let key1 = derive_key_from_password("mypassword", &salt).unwrap();
        let key2 = derive_key_from_password("mypassword", &salt).unwrap();

        // Same password + salt = same key
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_different_passwords() {
        let salt = generate_salt();
        let key1 = derive_key_from_password("password1", &salt).unwrap();
        let key2 = derive_key_from_password("password2", &salt).unwrap();

        // Different passwords = different keys
        assert_ne!(key1, key2);
    }

    #[cfg(feature = "async")]
    #[tokio::test]
    async fn test_key_manager_with_database() {
        // Create in-memory database
        let pool = SqlitePool::connect(":memory:").await.unwrap();

        // Create encryption_keys table
        sqlx::query(
            r#"
            CREATE TABLE encryption_keys (
                id TEXT PRIMARY KEY,
                version INTEGER NOT NULL UNIQUE,
                key_material TEXT NOT NULL,
                is_active INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        // Create manager with database
        let manager = KeyManager::with_database(pool.clone()).await.unwrap();

        // Should have generated initial key
        assert_eq!(manager.current_version(), 1);

        // Verify key was saved to database
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM encryption_keys")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(count, 1);
    }

    #[cfg(feature = "async")]
    #[tokio::test]
    async fn test_load_existing_keys() {
        // Create in-memory database
        let pool = SqlitePool::connect(":memory:").await.unwrap();

        // Create encryption_keys table
        sqlx::query(
            r#"
            CREATE TABLE encryption_keys (
                id TEXT PRIMARY KEY,
                version INTEGER NOT NULL UNIQUE,
                key_material TEXT NOT NULL,
                is_active INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        // Insert a test key
        let test_key = [42u8; 32];
        let key_material = BASE64.encode(test_key);
        let id = Uuid::new_v4().to_string();

        sqlx::query(
            "INSERT INTO encryption_keys (id, version, key_material, is_active) VALUES (?, 1, ?, 1)"
        )
        .bind(&id)
        .bind(&key_material)
        .execute(&pool)
        .await
        .unwrap();

        // Create manager - should load existing key
        let manager = KeyManager::with_database(pool.clone()).await.unwrap();

        assert_eq!(manager.current_version(), 1);
        let (loaded_key, _) = manager.get_current_key().unwrap();
        assert_eq!(loaded_key, &test_key);
    }
}
