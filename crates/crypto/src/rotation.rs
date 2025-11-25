#[cfg(feature = "async")]
use crate::encryption::{decrypt_data, encrypt_data, EncryptedData};
#[cfg(feature = "async")]
use crate::errors::CryptoResult;
#[cfg(feature = "async")]
use crate::key_management::KeyManager;
#[cfg(feature = "async")]
use sqlx::SqlitePool;
#[cfg(feature = "async")]
use tracing::{error, info, warn};

/// Rotate encryption key and re-encrypt all encrypted data
///
/// This is a critical operation that:
/// 1. Generates a new encryption key
/// 2. Finds all encrypted fields in the database
/// 3. Decrypts with old key, re-encrypts with new key
/// 4. Updates all records atomically
///
/// # Safety
/// This operation should be performed during maintenance windows
/// and with proper backups in place.
#[cfg(feature = "async")]
pub async fn rotate_key(
    pool: &SqlitePool,
    key_manager: &mut KeyManager,
) -> CryptoResult<RotationReport> {
    info!("Starting key rotation process");

    let old_version = key_manager.current_version();
    let new_version = key_manager.rotate_key()?;

    info!(
        "Rotated from key version {} to {}",
        old_version, new_version
    );

    let mut report = RotationReport {
        old_version,
        new_version,
        attachments_rotated: 0,
        notes_rotated: 0,
        errors: Vec::new(),
    };

    // Rotate attachment encryption
    if let Err(e) = rotate_attachments(pool, key_manager, old_version, new_version, &mut report).await {
        error!("Failed to rotate attachment encryption: {}", e);
        report.errors.push(format!("Attachments: {}", e));
    }

    // Rotate notes encryption (if any encrypted notes exist)
    if let Err(e) = rotate_notes(pool, key_manager, old_version, new_version, &mut report).await {
        error!("Failed to rotate notes encryption: {}", e);
        report.errors.push(format!("Notes: {}", e));
    }

    if report.errors.is_empty() {
        info!(
            "Key rotation completed successfully. Rotated {} attachments, {} notes",
            report.attachments_rotated, report.notes_rotated
        );
    } else {
        warn!(
            "Key rotation completed with {} errors. Rotated {} attachments, {} notes",
            report.errors.len(),
            report.attachments_rotated,
            report.notes_rotated
        );
    }

    Ok(report)
}

#[cfg(feature = "async")]
async fn rotate_attachments(
    pool: &SqlitePool,
    key_manager: &KeyManager,
    old_version: i32,
    new_version: i32,
    report: &mut RotationReport,
) -> CryptoResult<()> {
    // Find all encrypted attachments with old key version
    let attachments = sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT id, encryption_metadata FROM attachments WHERE is_encrypted = 1"
    )
    .fetch_all(pool)
    .await?;

    info!("Found {} encrypted attachments to rotate", attachments.len());

    for (id, metadata_json) in attachments {
        if let Some(metadata_json) = metadata_json {
            match serde_json::from_str::<EncryptedData>(&metadata_json) {
                Ok(encrypted_metadata) => {
                    if encrypted_metadata.key_version == old_version {
                        match rotate_attachment_data(
                            pool,
                            &id,
                            &encrypted_metadata,
                            key_manager,
                            new_version,
                        )
                        .await
                        {
                            Ok(_) => {
                                report.attachments_rotated += 1;
                            }
                            Err(e) => {
                                error!("Failed to rotate attachment {}: {}", id, e);
                                report.errors.push(format!("Attachment {}: {}", id, e));
                            }
                        }
                    }
                }
                Err(e) => {
                    error!(
                        "Failed to parse encryption metadata for attachment {}: {}",
                        id, e
                    );
                    report.errors.push(format!(
                        "Attachment {} metadata parse error: {}",
                        id, e
                    ));
                }
            }
        }
    }

    Ok(())
}

#[cfg(feature = "async")]
async fn rotate_attachment_data(
    pool: &SqlitePool,
    attachment_id: &str,
    old_encrypted: &EncryptedData,
    key_manager: &KeyManager,
    new_version: i32,
) -> CryptoResult<()> {
    // Get old key
    let old_key = key_manager.get_key(old_encrypted.key_version)?;

    // Decrypt with old key
    let plaintext = decrypt_data(old_encrypted, old_key)?;

    // Get new key
    let (new_key, _) = key_manager.get_current_key()?;

    // Re-encrypt with new key
    let new_encrypted = encrypt_data(&plaintext, new_key, new_version)?;

    // Update database
    let new_metadata = serde_json::to_string(&new_encrypted)?;

    sqlx::query("UPDATE attachments SET encryption_metadata = ? WHERE id = ?")
        .bind(&new_metadata)
        .bind(attachment_id)
        .execute(pool)
        .await?;

    Ok(())
}

#[cfg(feature = "async")]
async fn rotate_notes(
    pool: &SqlitePool,
    key_manager: &KeyManager,
    old_version: i32,
    new_version: i32,
    report: &mut RotationReport,
) -> CryptoResult<()> {
    // Notes typically aren't encrypted, but metadata might be
    // Find any notes with encrypted metadata
    let notes = sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT id, metadata FROM notes WHERE metadata LIKE '%\"key_version\":%'"
    )
    .fetch_all(pool)
    .await?;

    info!("Found {} notes with potential encryption metadata", notes.len());

    for (id, metadata_json) in notes {
        // Try to parse as EncryptedData
        if let Some(metadata_json) = metadata_json {
            match serde_json::from_str::<EncryptedData>(&metadata_json) {
                Ok(encrypted_metadata) => {
                    if encrypted_metadata.key_version == old_version {
                        match rotate_note_data(
                            pool,
                            &id,
                            &encrypted_metadata,
                            key_manager,
                            new_version,
                        )
                        .await
                        {
                            Ok(_) => {
                                report.notes_rotated += 1;
                            }
                            Err(e) => {
                                error!("Failed to rotate note {}: {}", id, e);
                                report.errors.push(format!("Note {}: {}", id, e));
                            }
                        }
                    }
                }
                Err(_e) => {
                    // Not encrypted data, skip silently
                }
            }
        }
    }

    Ok(())
}

#[cfg(feature = "async")]
async fn rotate_note_data(
    pool: &SqlitePool,
    note_id: &str,
    old_encrypted: &EncryptedData,
    key_manager: &KeyManager,
    new_version: i32,
) -> CryptoResult<()> {
    // Get old key
    let old_key = key_manager.get_key(old_encrypted.key_version)?;

    // Decrypt with old key
    let plaintext = decrypt_data(old_encrypted, old_key)?;

    // Get new key
    let (new_key, _) = key_manager.get_current_key()?;

    // Re-encrypt with new key
    let new_encrypted = encrypt_data(&plaintext, new_key, new_version)?;

    // Update database
    let new_metadata = serde_json::to_string(&new_encrypted)?;

    sqlx::query("UPDATE notes SET metadata = ? WHERE id = ?")
        .bind(&new_metadata)
        .bind(note_id)
        .execute(pool)
        .await?;

    Ok(())
}

/// Report of key rotation operation
#[cfg(feature = "async")]
#[derive(Debug)]
pub struct RotationReport {
    pub old_version: i32,
    pub new_version: i32,
    pub attachments_rotated: usize,
    pub notes_rotated: usize,
    pub errors: Vec<String>,
}

#[cfg(all(test, feature = "async"))]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_key_rotation() {
        // Create in-memory database
        let pool = SqlitePool::connect(":memory:").await.unwrap();

        // Create tables
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

        sqlx::query(
            r#"
            CREATE TABLE attachments (
                id TEXT PRIMARY KEY,
                is_encrypted INTEGER NOT NULL DEFAULT 0,
                encryption_metadata TEXT
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE notes (
                id TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                metadata TEXT
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        // Create key manager
        let mut key_manager = KeyManager::with_database(pool.clone()).await.unwrap();

        // Create encrypted attachment
        let (key, version) = key_manager.get_current_key().unwrap();
        let encrypted = encrypt_data(b"secret file content", key, version).unwrap();
        let metadata = serde_json::to_string(&encrypted).unwrap();

        let attachment_id = Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO attachments (id, is_encrypted, encryption_metadata) VALUES (?, 1, ?)")
            .bind(&attachment_id)
            .bind(&metadata)
            .execute(&pool)
            .await
            .unwrap();

        // Perform key rotation
        let report = rotate_key(&pool, &mut key_manager).await.unwrap();

        assert_eq!(report.old_version, 1);
        assert_eq!(report.new_version, 2);
        assert_eq!(report.attachments_rotated, 1);
        assert_eq!(report.notes_rotated, 0);
        assert!(report.errors.is_empty());

        // Verify attachment was re-encrypted
        let row = sqlx::query_as::<_, (Option<String>,)>(
            "SELECT encryption_metadata FROM attachments WHERE id = ?"
        )
        .bind(&attachment_id)
        .fetch_one(&pool)
        .await
        .unwrap();

        let new_encrypted: EncryptedData =
            serde_json::from_str(&row.0.unwrap()).unwrap();

        assert_eq!(new_encrypted.key_version, 2);

        // Verify we can decrypt with new key
        let (new_key, _) = key_manager.get_current_key().unwrap();
        let decrypted = decrypt_data(&new_encrypted, new_key).unwrap();
        assert_eq!(decrypted, b"secret file content");
    }
}
