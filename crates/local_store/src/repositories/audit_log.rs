use crate::db::DbPool;
use core_domain::{AuditAction, AuditLog, DomainError, DomainResult, ShareEntityType};
use uuid::Uuid;

pub struct AuditLogRepository<'a> {
    pool: &'a DbPool,
}

impl<'a> AuditLogRepository<'a> {
    pub fn new(pool: &'a DbPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, log: &AuditLog) -> DomainResult<()> {
        let entity_type_str = match log.entity_type {
            ShareEntityType::Contact => "Contact",
            ShareEntityType::Project => "Project",
            ShareEntityType::Note => "Note",
            ShareEntityType::CalendarEvent => "CalendarEvent",
            ShareEntityType::Group => "Group",
            ShareEntityType::Concept => "Concept",
            ShareEntityType::Pick => "Pick",
            ShareEntityType::Location => "Location",
        };

        let action_str = match log.action {
            AuditAction::Create => "Create",
            AuditAction::Read => "Read",
            AuditAction::Update => "Update",
            AuditAction::Delete => "Delete",
            AuditAction::Share => "Share",
            AuditAction::Unshare => "Unshare",
            AuditAction::AcceptShare => "AcceptShare",
        };

        sqlx::query(
            "INSERT INTO audit_logs (id, entity_type, entity_id, action, user_id, changes, ip_address, user_agent, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(log.id.to_string())
        .bind(entity_type_str)
        .bind(log.entity_id.to_string())
        .bind(action_str)
        .bind(log.user_id.to_string())
        .bind(serde_json::to_string(&log.changes).unwrap())
        .bind(&log.ip_address)
        .bind(&log.user_agent)
        .bind(log.created_at.to_rfc3339())
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn get_by_id(&self, id: Uuid) -> DomainResult<AuditLog> {
        let row = sqlx::query_as::<_, AuditLogRow>(
            "SELECT id, entity_type, entity_id, action, user_id, changes, ip_address, user_agent, created_at
             FROM audit_logs WHERE id = ?"
        )
        .bind(id.to_string())
        .fetch_optional(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?
        .ok_or_else(|| DomainError::NotFound(format!("AuditLog {}", id)))?;

        Ok(row.into())
    }

    pub async fn list_by_entity(
        &self,
        entity_type: ShareEntityType,
        entity_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> DomainResult<Vec<AuditLog>> {
        let entity_type_str = match entity_type {
            ShareEntityType::Contact => "Contact",
            ShareEntityType::Project => "Project",
            ShareEntityType::Note => "Note",
            ShareEntityType::CalendarEvent => "CalendarEvent",
            ShareEntityType::Group => "Group",
            ShareEntityType::Concept => "Concept",
            ShareEntityType::Pick => "Pick",
            ShareEntityType::Location => "Location",
        };

        let rows = sqlx::query_as::<_, AuditLogRow>(
            "SELECT id, entity_type, entity_id, action, user_id, changes, ip_address, user_agent, created_at
             FROM audit_logs WHERE entity_type = ? AND entity_id = ? ORDER BY created_at DESC LIMIT ? OFFSET ?"
        )
        .bind(entity_type_str)
        .bind(entity_id.to_string())
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_by_user(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> DomainResult<Vec<AuditLog>> {
        let rows = sqlx::query_as::<_, AuditLogRow>(
            "SELECT id, entity_type, entity_id, action, user_id, changes, ip_address, user_agent, created_at
             FROM audit_logs WHERE user_id = ? ORDER BY created_at DESC LIMIT ? OFFSET ?"
        )
        .bind(user_id.to_string())
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn delete(&self, id: Uuid) -> DomainResult<()> {
        sqlx::query("DELETE FROM audit_logs WHERE id = ?")
            .bind(id.to_string())
            .execute(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct AuditLogRow {
    id: String,
    entity_type: String,
    entity_id: String,
    action: String,
    user_id: String,
    changes: String,
    ip_address: Option<String>,
    user_agent: Option<String>,
    created_at: String,
}

impl From<AuditLogRow> for AuditLog {
    fn from(row: AuditLogRow) -> Self {
        let entity_type = match row.entity_type.as_str() {
            "Contact" => ShareEntityType::Contact,
            "Project" => ShareEntityType::Project,
            "Note" => ShareEntityType::Note,
            "CalendarEvent" => ShareEntityType::CalendarEvent,
            "Group" => ShareEntityType::Group,
            "Concept" => ShareEntityType::Concept,
            "Pick" => ShareEntityType::Pick,
            "Location" => ShareEntityType::Location,
            _ => ShareEntityType::Contact,
        };

        let action = match row.action.as_str() {
            "Create" => AuditAction::Create,
            "Read" => AuditAction::Read,
            "Update" => AuditAction::Update,
            "Delete" => AuditAction::Delete,
            "Share" => AuditAction::Share,
            "Unshare" => AuditAction::Unshare,
            "AcceptShare" => AuditAction::AcceptShare,
            _ => AuditAction::Read,
        };

        Self {
            id: Uuid::parse_str(&row.id).unwrap(),
            entity_type,
            entity_id: Uuid::parse_str(&row.entity_id).unwrap(),
            action,
            user_id: Uuid::parse_str(&row.user_id).unwrap(),
            changes: serde_json::from_str(&row.changes).unwrap_or(serde_json::json!({})),
            ip_address: row.ip_address,
            user_agent: row.user_agent,
            created_at: chrono::DateTime::parse_from_rfc3339(&row.created_at)
                .unwrap()
                .with_timezone(&chrono::Utc),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use sqlx::sqlite::SqlitePoolOptions;

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

    #[tokio::test]
    async fn test_create_and_get_audit_log() {
        let pool = setup_test_db().await;
        let repo = AuditLogRepository::new(&pool);

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

        let log = AuditLog {
            id: Uuid::new_v4(),
            entity_type: ShareEntityType::Contact,
            entity_id: contact_id,
            action: AuditAction::Update,
            user_id,
            changes: serde_json::json!({"field": "email", "old": null, "new": "john@example.com"}),
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: Some("TestAgent/1.0".to_string()),
            created_at: Utc::now(),
        };

        repo.create(&log).await.unwrap();

        let retrieved = repo.get_by_id(log.id).await.unwrap();
        assert_eq!(retrieved.user_id, user_id);
        assert_eq!(retrieved.ip_address, Some("127.0.0.1".to_string()));
    }

    #[tokio::test]
    async fn test_list_by_entity() {
        let pool = setup_test_db().await;
        let repo = AuditLogRepository::new(&pool);

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

        let log1 = AuditLog {
            id: Uuid::new_v4(),
            entity_type: ShareEntityType::Contact,
            entity_id: contact_id,
            action: AuditAction::Create,
            user_id,
            changes: serde_json::json!({}),
            ip_address: None,
            user_agent: None,
            created_at: Utc::now(),
        };

        let log2 = AuditLog {
            id: Uuid::new_v4(),
            entity_type: ShareEntityType::Contact,
            entity_id: contact_id,
            action: AuditAction::Update,
            user_id,
            changes: serde_json::json!({"email": "new@example.com"}),
            ip_address: None,
            user_agent: None,
            created_at: Utc::now(),
        };

        repo.create(&log1).await.unwrap();
        repo.create(&log2).await.unwrap();

        let logs = repo
            .list_by_entity(ShareEntityType::Contact, contact_id, 10, 0)
            .await
            .unwrap();
        assert_eq!(logs.len(), 2);
    }
}
