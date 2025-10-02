use core_domain::{Attachment, AttachmentEntityType, DomainResult, DomainError};
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

pub struct AttachmentRepository<'a> {
    pool: &'a Pool<Sqlite>,
}

impl<'a> AttachmentRepository<'a> {
    pub fn new(pool: &'a Pool<Sqlite>) -> Self {
        Self { pool }
    }

    pub async fn create(&self, attachment: &Attachment) -> DomainResult<()> {
        let entity_type_str = match attachment.entity_type {
            AttachmentEntityType::Contact => "Contact",
            AttachmentEntityType::Project => "Project",
            AttachmentEntityType::Note => "Note",
            AttachmentEntityType::CalendarEvent => "CalendarEvent",
            AttachmentEntityType::Communication => "Communication",
        };

        let scan_status_str = format!("{:?}", attachment.scan_status);

        sqlx::query(
            "INSERT INTO attachments (id, filename, content_type, size_bytes, storage_path, thumbnail_path, entity_type, entity_id, uploaded_by, checksum, encrypted, scan_status, scan_details, metadata, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(attachment.id.to_string())
        .bind(&attachment.filename)
        .bind(&attachment.content_type)
        .bind(attachment.size_bytes)
        .bind(&attachment.storage_path)
        .bind(&attachment.thumbnail_path)
        .bind(entity_type_str)
        .bind(attachment.entity_id.to_string())
        .bind(attachment.uploaded_by.to_string())
        .bind(&attachment.checksum)
        .bind(attachment.encrypted as i32)
        .bind(scan_status_str)
        .bind(&attachment.scan_details)
        .bind(serde_json::to_string(&attachment.metadata).unwrap())
        .bind(attachment.created_at.to_rfc3339())
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn get_by_id(&self, id: Uuid) -> DomainResult<Attachment> {
        let row = sqlx::query_as::<_, AttachmentRow>(
            "SELECT id, filename, content_type, size_bytes, storage_path, thumbnail_path, entity_type, entity_id, uploaded_by, checksum, encrypted, scan_status, scan_details, metadata, created_at
             FROM attachments WHERE id = ?"
        )
        .bind(id.to_string())
        .fetch_optional(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?
        .ok_or_else(|| DomainError::NotFound(format!("Attachment {}", id)))?;

        Ok(row.into())
    }

    pub async fn list_by_entity(&self, entity_type: AttachmentEntityType, entity_id: Uuid) -> DomainResult<Vec<Attachment>> {
        let entity_type_str = match entity_type {
            AttachmentEntityType::Contact => "Contact",
            AttachmentEntityType::Project => "Project",
            AttachmentEntityType::Note => "Note",
            AttachmentEntityType::CalendarEvent => "CalendarEvent",
            AttachmentEntityType::Communication => "Communication",
        };

        let rows = sqlx::query_as::<_, AttachmentRow>(
            "SELECT id, filename, content_type, size_bytes, storage_path, thumbnail_path, entity_type, entity_id, uploaded_by, checksum, encrypted, scan_status, scan_details, metadata, created_at
             FROM attachments WHERE entity_type = ? AND entity_id = ? ORDER BY created_at DESC"
        )
        .bind(entity_type_str)
        .bind(entity_id.to_string())
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update(&self, attachment: &Attachment) -> DomainResult<()> {
        let entity_type_str = match attachment.entity_type {
            AttachmentEntityType::Contact => "Contact",
            AttachmentEntityType::Project => "Project",
            AttachmentEntityType::Note => "Note",
            AttachmentEntityType::CalendarEvent => "CalendarEvent",
            AttachmentEntityType::Communication => "Communication",
        };

        let scan_status_str = format!("{:?}", attachment.scan_status);

        sqlx::query(
            "UPDATE attachments SET filename = ?, content_type = ?, size_bytes = ?, storage_path = ?, thumbnail_path = ?, entity_type = ?, entity_id = ?, scan_status = ?, scan_details = ?, metadata = ?
             WHERE id = ?"
        )
        .bind(&attachment.filename)
        .bind(&attachment.content_type)
        .bind(attachment.size_bytes)
        .bind(&attachment.storage_path)
        .bind(&attachment.thumbnail_path)
        .bind(entity_type_str)
        .bind(attachment.entity_id.to_string())
        .bind(scan_status_str)
        .bind(&attachment.scan_details)
        .bind(serde_json::to_string(&attachment.metadata).unwrap())
        .bind(attachment.id.to_string())
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn delete(&self, id: Uuid) -> DomainResult<()> {
        sqlx::query("DELETE FROM attachments WHERE id = ?")
            .bind(id.to_string())
            .execute(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct AttachmentRow {
    id: String,
    filename: String,
    content_type: String,
    size_bytes: i64,
    storage_path: String,
    thumbnail_path: Option<String>,
    entity_type: String,
    entity_id: String,
    uploaded_by: String,
    checksum: String,
    encrypted: i32,
    scan_status: String,
    scan_details: Option<String>,
    metadata: String,
    created_at: String,
}

impl From<AttachmentRow> for Attachment {
    fn from(row: AttachmentRow) -> Self {
        use core_domain::ScanStatus;

        let entity_type = match row.entity_type.as_str() {
            "Contact" => AttachmentEntityType::Contact,
            "Project" => AttachmentEntityType::Project,
            "Note" => AttachmentEntityType::Note,
            "CalendarEvent" => AttachmentEntityType::CalendarEvent,
            "Communication" => AttachmentEntityType::Communication,
            _ => AttachmentEntityType::Contact,
        };

        let scan_status = match row.scan_status.as_str() {
            "Pending" => ScanStatus::Pending,
            "Clean" => ScanStatus::Clean,
            "Infected" => ScanStatus::Infected,
            "Error" => ScanStatus::Error,
            _ => ScanStatus::Pending,
        };

        Self {
            id: Uuid::parse_str(&row.id).unwrap(),
            filename: row.filename,
            content_type: row.content_type,
            size_bytes: row.size_bytes,
            storage_path: row.storage_path,
            thumbnail_path: row.thumbnail_path,
            entity_type,
            entity_id: Uuid::parse_str(&row.entity_id).unwrap(),
            uploaded_by: Uuid::parse_str(&row.uploaded_by).unwrap(),
            checksum: row.checksum,
            encrypted: row.encrypted != 0,
            scan_status,
            scan_details: row.scan_details,
            metadata: serde_json::from_str(&row.metadata).unwrap_or(serde_json::json!({})),
            created_at: chrono::DateTime::parse_from_rfc3339(&row.created_at).unwrap().with_timezone(&chrono::Utc),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup_test_db() -> Pool<Sqlite> {
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

    #[tokio::test]
    async fn test_create_and_get_attachment() {
        let pool = setup_test_db().await;
        let repo = AttachmentRepository::new(&pool);

        let user_id = Uuid::new_v4();
        sqlx::query("INSERT INTO users (id, email, name, password_hash, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(user_id.to_string())
            .bind("test@example.com")
            .bind("Test")
            .bind("hash")
            .bind(Utc::now().to_rfc3339())
            .bind(Utc::now().to_rfc3339())
            .execute(&pool)
            .await
            .unwrap();

        let contact_id = Uuid::new_v4();
        sqlx::query("INSERT INTO contacts (id, first_name, created_at, updated_at, created_by, version) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(contact_id.to_string())
            .bind("John")
            .bind(Utc::now().to_rfc3339())
            .bind(Utc::now().to_rfc3339())
            .bind(user_id.to_string())
            .bind(1)
            .execute(&pool)
            .await
            .unwrap();

        let attachment = Attachment {
            id: Uuid::new_v4(),
            filename: "document.pdf".to_string(),
            content_type: "application/pdf".to_string(),
            size_bytes: 1024,
            storage_path: "/uploads/doc.pdf".to_string(),
            thumbnail_path: Some("/thumbs/doc.jpg".to_string()),
            entity_type: AttachmentEntityType::Contact,
            entity_id: contact_id,
            uploaded_by: user_id,
            checksum: "abc123".to_string(),
            encrypted: false,
            scan_status: core_domain::ScanStatus::Clean,
            scan_details: None,
            metadata: serde_json::json!({"original_name": "document.pdf"}),
            created_at: Utc::now(),
        };

        repo.create(&attachment).await.unwrap();

        let retrieved = repo.get_by_id(attachment.id).await.unwrap();
        assert_eq!(retrieved.filename, attachment.filename);
        assert_eq!(retrieved.size_bytes, 1024);
    }

    #[tokio::test]
    async fn test_list_by_entity() {
        let pool = setup_test_db().await;
        let repo = AttachmentRepository::new(&pool);

        let user_id = Uuid::new_v4();
        sqlx::query("INSERT INTO users (id, email, name, password_hash, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(user_id.to_string())
            .bind("test@example.com")
            .bind("Test")
            .bind("hash")
            .bind(Utc::now().to_rfc3339())
            .bind(Utc::now().to_rfc3339())
            .execute(&pool)
            .await
            .unwrap();

        let contact_id = Uuid::new_v4();
        sqlx::query("INSERT INTO contacts (id, first_name, created_at, updated_at, created_by, version) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(contact_id.to_string())
            .bind("John")
            .bind(Utc::now().to_rfc3339())
            .bind(Utc::now().to_rfc3339())
            .bind(user_id.to_string())
            .bind(1)
            .execute(&pool)
            .await
            .unwrap();

        let attachment1 = Attachment {
            id: Uuid::new_v4(),
            filename: "doc1.pdf".to_string(),
            content_type: "application/pdf".to_string(),
            size_bytes: 1024,
            storage_path: "/uploads/doc1.pdf".to_string(),
            thumbnail_path: None,
            entity_type: AttachmentEntityType::Contact,
            entity_id: contact_id,
            uploaded_by: user_id,
            checksum: "checksum1".to_string(),
            encrypted: false,
            scan_status: core_domain::ScanStatus::Pending,
            scan_details: None,
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
        };

        let attachment2 = Attachment {
            id: Uuid::new_v4(),
            filename: "doc2.pdf".to_string(),
            content_type: "application/pdf".to_string(),
            size_bytes: 2048,
            storage_path: "/uploads/doc2.pdf".to_string(),
            thumbnail_path: None,
            entity_type: AttachmentEntityType::Contact,
            entity_id: contact_id,
            uploaded_by: user_id,
            checksum: "checksum2".to_string(),
            encrypted: false,
            scan_status: core_domain::ScanStatus::Pending,
            scan_details: None,
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
        };

        repo.create(&attachment1).await.unwrap();
        repo.create(&attachment2).await.unwrap();

        let attachments = repo.list_by_entity(AttachmentEntityType::Contact, contact_id).await.unwrap();
        assert_eq!(attachments.len(), 2);
    }
}

#[cfg(test)]
#[path = "attachment_tests.rs"]
mod attachment_tests;