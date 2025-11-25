//! Integration tests for the crypto crate

use crate::{
    decrypt_data, encrypt_data, generate_salt, derive_key_from_password, KeyManager,
};

#[cfg(feature = "async")]
use uuid::Uuid;

#[test]
fn test_end_to_end_encryption() {
    // Create key manager
    let manager = KeyManager::new().unwrap();
    let (key, version) = manager.get_current_key().unwrap();

    // Encrypt some data
    let plaintext = b"This is a secret message!";
    let encrypted = encrypt_data(plaintext, key, version).unwrap();

    // Decrypt it back
    let decrypted = decrypt_data(&encrypted, key).unwrap();

    assert_eq!(plaintext, decrypted.as_slice());
}

#[test]
fn test_key_rotation_scenario() {
    // Create key manager
    let mut manager = KeyManager::new().unwrap();

    // Encrypt with first key
    let (key1, version1) = manager.get_current_key().unwrap();
    let plaintext = b"Old data";
    let encrypted_old = encrypt_data(plaintext, key1, version1).unwrap();

    // Rotate key
    let _version2 = manager.rotate_key().unwrap();

    // Encrypt new data with new key
    let (key2, version2) = manager.get_current_key().unwrap();
    let encrypted_new = encrypt_data(b"New data", key2, version2).unwrap();

    // Should still be able to decrypt old data
    let old_key = manager.get_key(version1).unwrap();
    let decrypted_old = decrypt_data(&encrypted_old, old_key).unwrap();
    assert_eq!(plaintext, decrypted_old.as_slice());

    // And decrypt new data
    let decrypted_new = decrypt_data(&encrypted_new, key2).unwrap();
    assert_eq!(b"New data", decrypted_new.as_slice());
}

#[test]
fn test_password_based_encryption() {
    let password = "my_secure_password";
    let salt = generate_salt();

    // Derive key from password
    let key = derive_key_from_password(password, &salt).unwrap();

    // Encrypt data
    let plaintext = b"Password-protected data";
    let encrypted = encrypt_data(plaintext, &key, 1).unwrap();

    // Later: derive same key from same password
    let key2 = derive_key_from_password(password, &salt).unwrap();
    assert_eq!(key, key2);

    // Decrypt data
    let decrypted = decrypt_data(&encrypted, &key2).unwrap();
    assert_eq!(plaintext, decrypted.as_slice());
}

#[test]
fn test_large_data_encryption() {
    let manager = KeyManager::new().unwrap();
    let (key, version) = manager.get_current_key().unwrap();

    // Test with 1MB of data
    let plaintext = vec![42u8; 1024 * 1024];
    let encrypted = encrypt_data(&plaintext, key, version).unwrap();
    let decrypted = decrypt_data(&encrypted, key).unwrap();

    assert_eq!(plaintext, decrypted);
}

#[test]
fn test_multiple_encryptions_different_nonces() {
    let manager = KeyManager::new().unwrap();
    let (key, version) = manager.get_current_key().unwrap();

    let plaintext = b"Same message";

    // Encrypt multiple times
    let e1 = encrypt_data(plaintext, key, version).unwrap();
    let e2 = encrypt_data(plaintext, key, version).unwrap();
    let e3 = encrypt_data(plaintext, key, version).unwrap();

    // All should have different nonces and ciphertexts
    assert_ne!(e1.nonce, e2.nonce);
    assert_ne!(e2.nonce, e3.nonce);
    assert_ne!(e1.ciphertext, e2.ciphertext);

    // But all should decrypt to same plaintext
    let d1 = decrypt_data(&e1, key).unwrap();
    let d2 = decrypt_data(&e2, key).unwrap();
    let d3 = decrypt_data(&e3, key).unwrap();

    assert_eq!(d1, plaintext);
    assert_eq!(d2, plaintext);
    assert_eq!(d3, plaintext);
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_async_key_manager() {
    use sqlx::SqlitePool;

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

    // Use it to encrypt/decrypt
    let (key, version) = manager.get_current_key().unwrap();
    let plaintext = b"Database-backed encryption";
    let encrypted = encrypt_data(plaintext, key, version).unwrap();
    let decrypted = decrypt_data(&encrypted, key).unwrap();

    assert_eq!(plaintext, decrypted.as_slice());
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_full_rotation_workflow() {
    use crate::rotate_key;
    use sqlx::SqlitePool;
    use uuid::Uuid;

    // Create in-memory database with all required tables
    let pool = SqlitePool::connect(":memory:").await.unwrap();

    sqlx::query(
        r#"
        CREATE TABLE encryption_keys (
            id TEXT PRIMARY KEY,
            version INTEGER NOT NULL UNIQUE,
            key_material TEXT NOT NULL,
            is_active INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        CREATE TABLE attachments (
            id TEXT PRIMARY KEY,
            is_encrypted INTEGER NOT NULL DEFAULT 0,
            encryption_metadata TEXT
        );
        CREATE TABLE notes (
            id TEXT PRIMARY KEY,
            content TEXT NOT NULL,
            metadata TEXT
        );
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create key manager
    let mut manager = KeyManager::with_database(pool.clone()).await.unwrap();

    // Create some encrypted data
    let (key1, version1) = manager.get_current_key().unwrap();
    let encrypted1 = encrypt_data(b"file content", key1, version1).unwrap();
    let metadata1 = serde_json::to_string(&encrypted1).unwrap();

    let id = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO attachments (id, is_encrypted, encryption_metadata) VALUES (?, 1, ?)")
        .bind(&id)
        .bind(&metadata1)
        .execute(&pool)
        .await
        .unwrap();

    // Perform rotation
    let report = rotate_key(&pool, &mut manager).await.unwrap();

    assert_eq!(report.attachments_rotated, 1);
    assert!(report.errors.is_empty());

    // Verify data is still accessible with new key
    let row = sqlx::query_as::<_, (Option<String>,)>(
        "SELECT encryption_metadata FROM attachments WHERE id = ?"
    )
    .bind(&id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let new_encrypted: crate::EncryptedData =
        serde_json::from_str(&row.0.unwrap()).unwrap();

    let (new_key, _) = manager.get_current_key().unwrap();
    let decrypted = decrypt_data(&new_encrypted, new_key).unwrap();

    assert_eq!(decrypted, b"file content");
}
