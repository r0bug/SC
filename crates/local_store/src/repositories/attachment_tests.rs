#[cfg(test)]
mod attachment_integration_tests {
    use super::super::*;
    use crate::repositories::attachment::AttachmentRepository;
    use chrono::Utc;
    use core_domain::{Attachment, AttachmentEntityType, ScanStatus};
    use sqlx::sqlite::SqlitePoolOptions;
    use crate::db::DbPool;
    use uuid::Uuid;

    async fn setup_test_db() -> DbPool {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query(crate::migrations::SCHEMA)
            .execute(&pool)
            .await
            .unwrap();

        pool
    }

    async fn create_test_user(pool: &DbPool) -> Uuid {
        let user_id = Uuid::new_v4();
        sqlx::query("INSERT INTO users (id, email, name, password_hash, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(user_id.to_string())
            .bind("test@example.com")
            .bind("Test User")
            .bind("hash")
            .bind(Utc::now().to_rfc3339())
            .bind(Utc::now().to_rfc3339())
            .execute(pool)
            .await
            .unwrap();
        user_id
    }

    async fn create_test_contact(pool: &DbPool, user_id: Uuid) -> Uuid {
        let contact_id = Uuid::new_v4();
        sqlx::query("INSERT INTO contacts (id, first_name, created_at, updated_at, created_by, version) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(contact_id.to_string())
            .bind("John Doe")
            .bind(Utc::now().to_rfc3339())
            .bind(Utc::now().to_rfc3339())
            .bind(user_id.to_string())
            .bind(1)
            .execute(pool)
            .await
            .unwrap();
        contact_id
    }

    #[tokio::test]
    async fn test_attachment_create_and_retrieve() {
        let pool = setup_test_db().await;
        let repo = AttachmentRepository::new(&pool);
        let user_id = create_test_user(&pool).await;
        let contact_id = create_test_contact(&pool, user_id).await;

        let attachment = Attachment {
            id: Uuid::new_v4(),
            filename: "test.pdf".to_string(),
            content_type: "application/pdf".to_string(),
            size_bytes: 1024,
            storage_path: "/data/test.pdf".to_string(),
            thumbnail_path: None,
            entity_type: AttachmentEntityType::Contact,
            entity_id: contact_id,
            uploaded_by: user_id,
            checksum: "abc123".to_string(),
            encrypted: false,
            scan_status: ScanStatus::Clean,
            scan_details: None,
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
        };

        repo.create(&attachment).await.unwrap();

        let retrieved = repo.get_by_id(attachment.id).await.unwrap();
        assert_eq!(retrieved.filename, "test.pdf");
        assert_eq!(retrieved.size_bytes, 1024);
        assert_eq!(retrieved.checksum, "abc123");
        assert_eq!(retrieved.scan_status, ScanStatus::Clean);
    }

    #[tokio::test]
    async fn test_attachment_list_by_entity() {
        let pool = setup_test_db().await;
        let repo = AttachmentRepository::new(&pool);
        let user_id = create_test_user(&pool).await;
        let contact_id = create_test_contact(&pool, user_id).await;

        // Create multiple attachments
        for i in 1..=3 {
            let attachment = Attachment {
                id: Uuid::new_v4(),
                filename: format!("file{}.pdf", i),
                content_type: "application/pdf".to_string(),
                size_bytes: 1024 * i,
                storage_path: format!("/data/file{}.pdf", i),
                thumbnail_path: None,
                entity_type: AttachmentEntityType::Contact,
                entity_id: contact_id,
                uploaded_by: user_id,
                checksum: format!("checksum{}", i),
                encrypted: false,
                scan_status: ScanStatus::Pending,
                scan_details: None,
                metadata: serde_json::json!({}),
                created_at: Utc::now(),
            };
            repo.create(&attachment).await.unwrap();
        }

        let attachments = repo
            .list_by_entity(AttachmentEntityType::Contact, contact_id)
            .await
            .unwrap();

        assert_eq!(attachments.len(), 3);
        assert!(attachments[0].filename.starts_with("file"));
    }

    #[tokio::test]
    async fn test_attachment_scan_status_tracking() {
        let pool = setup_test_db().await;
        let repo = AttachmentRepository::new(&pool);
        let user_id = create_test_user(&pool).await;
        let contact_id = create_test_contact(&pool, user_id).await;

        // Create attachment with Pending status
        let mut attachment = Attachment {
            id: Uuid::new_v4(),
            filename: "suspicious.exe".to_string(),
            content_type: "application/x-msdownload".to_string(),
            size_bytes: 2048,
            storage_path: "/data/suspicious.exe".to_string(),
            thumbnail_path: None,
            entity_type: AttachmentEntityType::Contact,
            entity_id: contact_id,
            uploaded_by: user_id,
            checksum: "def456".to_string(),
            encrypted: false,
            scan_status: ScanStatus::Pending,
            scan_details: None,
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
        };

        repo.create(&attachment).await.unwrap();

        // Simulate scan completing with Infected status
        attachment.scan_status = ScanStatus::Infected;
        attachment.scan_details = Some("Malware detected: Trojan.Generic".to_string());
        repo.update(&attachment).await.unwrap();

        let retrieved = repo.get_by_id(attachment.id).await.unwrap();
        assert_eq!(retrieved.scan_status, ScanStatus::Infected);
        assert_eq!(
            retrieved.scan_details,
            Some("Malware detected: Trojan.Generic".to_string())
        );
    }

    #[tokio::test]
    async fn test_attachment_checksum_integrity() {
        let pool = setup_test_db().await;
        let repo = AttachmentRepository::new(&pool);
        let user_id = create_test_user(&pool).await;
        let contact_id = create_test_contact(&pool, user_id).await;

        let original_checksum = "a1b2c3d4e5f6";
        let attachment = Attachment {
            id: Uuid::new_v4(),
            filename: "important.doc".to_string(),
            content_type: "application/msword".to_string(),
            size_bytes: 4096,
            storage_path: "/data/important.doc".to_string(),
            thumbnail_path: None,
            entity_type: AttachmentEntityType::Contact,
            entity_id: contact_id,
            uploaded_by: user_id,
            checksum: original_checksum.to_string(),
            encrypted: false,
            scan_status: ScanStatus::Clean,
            scan_details: None,
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
        };

        repo.create(&attachment).await.unwrap();

        let retrieved = repo.get_by_id(attachment.id).await.unwrap();
        assert_eq!(retrieved.checksum, original_checksum);
    }

    #[tokio::test]
    async fn test_attachment_encryption_flag() {
        let pool = setup_test_db().await;
        let repo = AttachmentRepository::new(&pool);
        let user_id = create_test_user(&pool).await;
        let contact_id = create_test_contact(&pool, user_id).await;

        let attachment = Attachment {
            id: Uuid::new_v4(),
            filename: "secret.txt".to_string(),
            content_type: "text/plain".to_string(),
            size_bytes: 512,
            storage_path: "/data/secret.txt.enc".to_string(),
            thumbnail_path: None,
            entity_type: AttachmentEntityType::Contact,
            entity_id: contact_id,
            uploaded_by: user_id,
            checksum: "encrypted_checksum".to_string(),
            encrypted: true,
            scan_status: ScanStatus::Clean,
            scan_details: None,
            metadata: serde_json::json!({"encryption": "AES-256"}),
            created_at: Utc::now(),
        };

        repo.create(&attachment).await.unwrap();

        let retrieved = repo.get_by_id(attachment.id).await.unwrap();
        assert!(retrieved.encrypted);
        assert_eq!(
            retrieved
                .metadata
                .get("encryption")
                .and_then(|v| v.as_str()),
            Some("AES-256")
        );
    }

    #[tokio::test]
    async fn test_attachment_delete() {
        let pool = setup_test_db().await;
        let repo = AttachmentRepository::new(&pool);
        let user_id = create_test_user(&pool).await;
        let contact_id = create_test_contact(&pool, user_id).await;

        let attachment = Attachment {
            id: Uuid::new_v4(),
            filename: "delete_me.txt".to_string(),
            content_type: "text/plain".to_string(),
            size_bytes: 128,
            storage_path: "/data/delete_me.txt".to_string(),
            thumbnail_path: None,
            entity_type: AttachmentEntityType::Contact,
            entity_id: contact_id,
            uploaded_by: user_id,
            checksum: "xyz789".to_string(),
            encrypted: false,
            scan_status: ScanStatus::Clean,
            scan_details: None,
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
        };

        repo.create(&attachment).await.unwrap();

        // Verify it exists
        assert!(repo.get_by_id(attachment.id).await.is_ok());

        // Delete it
        repo.delete(attachment.id).await.unwrap();

        // Verify it's gone
        assert!(repo.get_by_id(attachment.id).await.is_err());
    }

    #[tokio::test]
    async fn test_attachment_multiple_entity_types() {
        let pool = setup_test_db().await;
        let repo = AttachmentRepository::new(&pool);
        let user_id = create_test_user(&pool).await;
        let contact_id = create_test_contact(&pool, user_id).await;

        // Create attachment for Contact
        let contact_attachment = Attachment {
            id: Uuid::new_v4(),
            filename: "contact_file.pdf".to_string(),
            content_type: "application/pdf".to_string(),
            size_bytes: 1024,
            storage_path: "/data/contact_file.pdf".to_string(),
            thumbnail_path: None,
            entity_type: AttachmentEntityType::Contact,
            entity_id: contact_id,
            uploaded_by: user_id,
            checksum: "contact_checksum".to_string(),
            encrypted: false,
            scan_status: ScanStatus::Clean,
            scan_details: None,
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
        };

        repo.create(&contact_attachment).await.unwrap();

        // Verify retrieval by entity type
        let contact_attachments = repo
            .list_by_entity(AttachmentEntityType::Contact, contact_id)
            .await
            .unwrap();

        assert_eq!(contact_attachments.len(), 1);
        assert_eq!(contact_attachments[0].filename, "contact_file.pdf");
    }
}
