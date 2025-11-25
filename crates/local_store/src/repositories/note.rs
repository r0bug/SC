use core_domain::{DomainError, DomainResult, Note};
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

pub struct NoteRepository<'a> {
    pool: &'a Pool<Sqlite>,
}

impl<'a> NoteRepository<'a> {
    pub fn new(pool: &'a Pool<Sqlite>) -> Self {
        Self { pool }
    }

    pub async fn create(&self, note: &Note) -> DomainResult<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        sqlx::query(
            "INSERT INTO notes (id, contact_id, project_id, title, content, created_at, updated_at, created_by, version, last_synced_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(note.id.to_string())
        .bind(note.contact_id.map(|id| id.to_string()))
        .bind(note.project_id.map(|id| id.to_string()))
        .bind(&note.title)
        .bind(&note.content)
        .bind(note.created_at.to_rfc3339())
        .bind(note.updated_at.to_rfc3339())
        .bind(note.created_by.to_string())
        .bind(note.version)
        .bind(note.last_synced_at.map(|d| d.to_rfc3339()))
        .execute(&mut *tx)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        // Note: attachment_ids are now stored, not created inline with notes
        // Attachments are managed separately via AttachmentRepository

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn get_by_id(&self, id: Uuid) -> DomainResult<Note> {
        let row = sqlx::query_as::<_, NoteRow>(
            "SELECT id, contact_id, project_id, title, content, created_at, updated_at, created_by, version, last_synced_at FROM notes WHERE id = ?"
        )
        .bind(id.to_string())
        .fetch_one(self.pool)
        .await
        .map_err(|e| DomainError::NotFound(format!("Note not found: {}", e)))?;

        let attachment_ids = vec![]; // Attachments managed separately
        Ok(row.into_note(attachment_ids))
    }

    pub async fn list_by_contact(&self, contact_id: Uuid, user_id: Uuid) -> DomainResult<Vec<Note>> {
        let rows = sqlx::query_as::<_, NoteRow>(
            "SELECT id, contact_id, project_id, title, content, created_at, updated_at, created_by, version, last_synced_at FROM notes WHERE contact_id = ? AND created_by = ? ORDER BY created_at DESC"
        )
        .bind(contact_id.to_string())
        .bind(user_id.to_string())
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        self.hydrate_notes(rows).await
    }

    async fn hydrate_notes(&self, rows: Vec<NoteRow>) -> DomainResult<Vec<Note>> {
        let mut notes = Vec::new();
        for row in rows {
            // Attachment IDs would be fetched from a junction table if needed
            // For now, using empty vec as attachments are managed separately
            let attachment_ids = vec![];

            notes.push(row.into_note(attachment_ids));
        }
        Ok(notes)
    }

    pub async fn update(&self, note: &Note) -> DomainResult<()> {
        sqlx::query(
            "UPDATE notes SET contact_id = ?, project_id = ?, title = ?, content = ?, updated_at = ? WHERE id = ?"
        )
        .bind(note.contact_id.map(|id| id.to_string()))
        .bind(note.project_id.map(|id| id.to_string()))
        .bind(&note.title)
        .bind(&note.content)
        .bind(note.updated_at.to_rfc3339())
        .bind(note.id.to_string())
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(())
    }

    pub async fn delete(&self, id: Uuid) -> DomainResult<()> {
        sqlx::query("DELETE FROM notes WHERE id = ?")
            .bind(id.to_string())
            .execute(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct NoteRow {
    id: String,
    contact_id: Option<String>,
    project_id: Option<String>,
    title: String,
    content: String,
    created_at: String,
    updated_at: String,
    created_by: String,
    version: i32,
    last_synced_at: Option<String>,
}

impl NoteRow {
    fn into_note(self, attachment_ids: Vec<Uuid>) -> Note {
        Note {
            id: Uuid::parse_str(&self.id).unwrap(),
            contact_id: self.contact_id.and_then(|s| Uuid::parse_str(&s).ok()),
            project_id: self.project_id.and_then(|s| Uuid::parse_str(&s).ok()),
            title: self.title,
            content: self.content,
            attachment_ids,
            created_at: chrono::DateTime::parse_from_rfc3339(&self.created_at)
                .unwrap()
                .with_timezone(&chrono::Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(&self.updated_at)
                .unwrap()
                .with_timezone(&chrono::Utc),
            created_by: Uuid::parse_str(&self.created_by).unwrap(),
            version: self.version,
            last_synced_at: self.last_synced_at.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|dt| dt.with_timezone(&chrono::Utc))
            }),
        }
    }
}

// AttachmentRow removed - attachments are now managed via AttachmentRepository
