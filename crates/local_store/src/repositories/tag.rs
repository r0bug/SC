use core_domain::{DomainError, DomainResult, Tag};
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

pub struct TagRepository<'a> {
    pool: &'a Pool<Sqlite>,
}

impl<'a> TagRepository<'a> {
    pub fn new(pool: &'a Pool<Sqlite>) -> Self {
        Self { pool }
    }

    pub async fn create(&self, tag: &Tag) -> DomainResult<()> {
        sqlx::query("INSERT INTO tags (id, name, color, created_at) VALUES (?, ?, ?, ?)")
            .bind(tag.id.to_string())
            .bind(&tag.name)
            .bind(&tag.color)
            .bind(tag.created_at.to_rfc3339())
            .execute(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(())
    }

    pub async fn list(&self) -> DomainResult<Vec<Tag>> {
        let rows = sqlx::query_as::<_, TagRow>(
            "SELECT id, name, color, created_at FROM tags ORDER BY name",
        )
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn add_to_contact(&self, contact_id: Uuid, tag_id: Uuid) -> DomainResult<()> {
        sqlx::query("INSERT OR IGNORE INTO contact_tags (contact_id, tag_id) VALUES (?, ?)")
            .bind(contact_id.to_string())
            .bind(tag_id.to_string())
            .execute(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(())
    }

    pub async fn update(&self, tag: &Tag) -> DomainResult<()> {
        sqlx::query("UPDATE tags SET name = ?, color = ? WHERE id = ?")
            .bind(&tag.name)
            .bind(&tag.color)
            .bind(tag.id.to_string())
            .execute(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(())
    }

    pub async fn delete(&self, id: Uuid) -> DomainResult<()> {
        // First delete associations
        sqlx::query("DELETE FROM contact_tags WHERE tag_id = ?")
            .bind(id.to_string())
            .execute(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        // Then delete the tag
        sqlx::query("DELETE FROM tags WHERE id = ?")
            .bind(id.to_string())
            .execute(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct TagRow {
    id: String,
    name: String,
    color: Option<String>,
    created_at: String,
}

impl From<TagRow> for Tag {
    fn from(row: TagRow) -> Self {
        Self {
            id: Uuid::parse_str(&row.id).unwrap(),
            name: row.name,
            color: row.color,
            created_at: chrono::DateTime::parse_from_rfc3339(&row.created_at)
                .unwrap()
                .with_timezone(&chrono::Utc),
        }
    }
}
