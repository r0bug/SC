use core_domain::{
    ConflictResolution, DomainError, DomainResult, ResolutionStrategy, ShareEntityType,
};
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

pub struct ConflictResolutionRepository<'a> {
    pool: &'a Pool<Sqlite>,
}

impl<'a> ConflictResolutionRepository<'a> {
    pub fn new(pool: &'a Pool<Sqlite>) -> Self {
        Self { pool }
    }

    pub async fn create(&self, resolution: &ConflictResolution) -> DomainResult<()> {
        let entity_type_str = match resolution.entity_type {
            ShareEntityType::Contact => "Contact",
            ShareEntityType::Project => "Project",
            ShareEntityType::Note => "Note",
            ShareEntityType::CalendarEvent => "CalendarEvent",
            ShareEntityType::Group => "Group",
        };

        let strategy_str = match resolution.resolution_strategy {
            ResolutionStrategy::LocalWins => "LocalWins",
            ResolutionStrategy::RemoteWins => "RemoteWins",
            ResolutionStrategy::Manual => "Manual",
            ResolutionStrategy::Merge => "Merge",
        };

        sqlx::query(
            "INSERT INTO conflict_resolutions (id, entity_type, entity_id, local_version, remote_version, local_updated_at, remote_updated_at, local_changes, remote_changes, resolution_strategy, resolved, resolved_at, resolved_by, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(resolution.id.to_string())
        .bind(entity_type_str)
        .bind(resolution.entity_id.to_string())
        .bind(resolution.local_version)
        .bind(resolution.remote_version)
        .bind(resolution.local_updated_at.to_rfc3339())
        .bind(resolution.remote_updated_at.to_rfc3339())
        .bind(serde_json::to_string(&resolution.local_changes).unwrap())
        .bind(serde_json::to_string(&resolution.remote_changes).unwrap())
        .bind(strategy_str)
        .bind(resolution.resolved as i32)
        .bind(resolution.resolved_at.map(|t| t.to_rfc3339()))
        .bind(resolution.resolved_by.map(|id| id.to_string()))
        .bind(resolution.created_at.to_rfc3339())
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn get_by_id(&self, id: Uuid) -> DomainResult<ConflictResolution> {
        let row = sqlx::query_as::<_, ConflictResolutionRow>(
            "SELECT id, entity_type, entity_id, local_version, remote_version, local_updated_at, remote_updated_at, local_changes, remote_changes, resolution_strategy, resolved, resolved_at, resolved_by, created_at
             FROM conflict_resolutions WHERE id = ?"
        )
        .bind(id.to_string())
        .fetch_optional(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?
        .ok_or_else(|| DomainError::NotFound(format!("ConflictResolution {}", id)))?;

        Ok(row.into())
    }

    pub async fn list_unresolved(
        &self,
        limit: i64,
        offset: i64,
    ) -> DomainResult<Vec<ConflictResolution>> {
        let rows = sqlx::query_as::<_, ConflictResolutionRow>(
            "SELECT id, entity_type, entity_id, local_version, remote_version, local_updated_at, remote_updated_at, local_changes, remote_changes, resolution_strategy, resolved, resolved_at, resolved_by, created_at
             FROM conflict_resolutions WHERE resolved = 0 ORDER BY created_at DESC LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_by_entity(
        &self,
        entity_type: ShareEntityType,
        entity_id: Uuid,
    ) -> DomainResult<Vec<ConflictResolution>> {
        let entity_type_str = match entity_type {
            ShareEntityType::Contact => "Contact",
            ShareEntityType::Project => "Project",
            ShareEntityType::Note => "Note",
            ShareEntityType::CalendarEvent => "CalendarEvent",
            ShareEntityType::Group => "Group",
        };

        let rows = sqlx::query_as::<_, ConflictResolutionRow>(
            "SELECT id, entity_type, entity_id, local_version, remote_version, local_updated_at, remote_updated_at, local_changes, remote_changes, resolution_strategy, resolved, resolved_at, resolved_by, created_at
             FROM conflict_resolutions WHERE entity_type = ? AND entity_id = ? ORDER BY created_at DESC"
        )
        .bind(entity_type_str)
        .bind(entity_id.to_string())
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn mark_resolved(&self, id: Uuid, resolved_by: Uuid) -> DomainResult<()> {
        sqlx::query(
            "UPDATE conflict_resolutions SET resolved = 1, resolved_at = ?, resolved_by = ? WHERE id = ?"
        )
        .bind(chrono::Utc::now().to_rfc3339())
        .bind(resolved_by.to_string())
        .bind(id.to_string())
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn update_strategy(
        &self,
        id: Uuid,
        strategy: ResolutionStrategy,
    ) -> DomainResult<()> {
        let strategy_str = match strategy {
            ResolutionStrategy::LocalWins => "LocalWins",
            ResolutionStrategy::RemoteWins => "RemoteWins",
            ResolutionStrategy::Manual => "Manual",
            ResolutionStrategy::Merge => "Merge",
        };

        sqlx::query("UPDATE conflict_resolutions SET resolution_strategy = ? WHERE id = ?")
            .bind(strategy_str)
            .bind(id.to_string())
            .execute(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn delete(&self, id: Uuid) -> DomainResult<()> {
        sqlx::query("DELETE FROM conflict_resolutions WHERE id = ?")
            .bind(id.to_string())
            .execute(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct ConflictResolutionRow {
    id: String,
    entity_type: String,
    entity_id: String,
    local_version: i32,
    remote_version: i32,
    local_updated_at: String,
    remote_updated_at: String,
    local_changes: String,
    remote_changes: String,
    resolution_strategy: String,
    resolved: i32,
    resolved_at: Option<String>,
    resolved_by: Option<String>,
    created_at: String,
}

impl From<ConflictResolutionRow> for ConflictResolution {
    fn from(row: ConflictResolutionRow) -> Self {
        let entity_type = match row.entity_type.as_str() {
            "Contact" => ShareEntityType::Contact,
            "Project" => ShareEntityType::Project,
            "Note" => ShareEntityType::Note,
            "CalendarEvent" => ShareEntityType::CalendarEvent,
            "Group" => ShareEntityType::Group,
            _ => ShareEntityType::Contact,
        };

        let resolution_strategy = match row.resolution_strategy.as_str() {
            "LocalWins" => ResolutionStrategy::LocalWins,
            "RemoteWins" => ResolutionStrategy::RemoteWins,
            "Manual" => ResolutionStrategy::Manual,
            "Merge" => ResolutionStrategy::Merge,
            _ => ResolutionStrategy::Manual,
        };

        Self {
            id: Uuid::parse_str(&row.id).unwrap(),
            entity_type,
            entity_id: Uuid::parse_str(&row.entity_id).unwrap(),
            local_version: row.local_version,
            remote_version: row.remote_version,
            local_updated_at: chrono::DateTime::parse_from_rfc3339(&row.local_updated_at)
                .unwrap()
                .with_timezone(&chrono::Utc),
            remote_updated_at: chrono::DateTime::parse_from_rfc3339(&row.remote_updated_at)
                .unwrap()
                .with_timezone(&chrono::Utc),
            local_changes: serde_json::from_str(&row.local_changes)
                .unwrap_or(serde_json::json!({})),
            remote_changes: serde_json::from_str(&row.remote_changes)
                .unwrap_or(serde_json::json!({})),
            resolution_strategy,
            resolved: row.resolved != 0,
            resolved_at: row.resolved_at.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|dt| dt.with_timezone(&chrono::Utc))
            }),
            resolved_by: row.resolved_by.and_then(|s| Uuid::parse_str(&s).ok()),
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
    async fn test_create_and_get_conflict() {
        let pool = setup_test_db().await;
        let repo = ConflictResolutionRepository::new(&pool);

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

        let conflict = ConflictResolution {
            id: Uuid::new_v4(),
            entity_type: ShareEntityType::Contact,
            entity_id: contact_id,
            local_version: 2,
            remote_version: 3,
            local_updated_at: Utc::now(),
            remote_updated_at: Utc::now(),
            local_changes: serde_json::json!({"email": "local@example.com"}),
            remote_changes: serde_json::json!({"email": "remote@example.com"}),
            resolution_strategy: ResolutionStrategy::Manual,
            resolved: false,
            resolved_at: None,
            resolved_by: None,
            created_at: Utc::now(),
        };

        repo.create(&conflict).await.unwrap();

        let retrieved = repo.get_by_id(conflict.id).await.unwrap();
        assert_eq!(retrieved.local_version, 2);
        assert_eq!(retrieved.remote_version, 3);
        assert!(!retrieved.resolved);
    }

    #[tokio::test]
    async fn test_mark_resolved() {
        let pool = setup_test_db().await;
        let repo = ConflictResolutionRepository::new(&pool);

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

        let conflict = ConflictResolution {
            id: Uuid::new_v4(),
            entity_type: ShareEntityType::Contact,
            entity_id: contact_id,
            local_version: 2,
            remote_version: 3,
            local_updated_at: Utc::now(),
            remote_updated_at: Utc::now(),
            local_changes: serde_json::json!({}),
            remote_changes: serde_json::json!({}),
            resolution_strategy: ResolutionStrategy::LocalWins,
            resolved: false,
            resolved_at: None,
            resolved_by: None,
            created_at: Utc::now(),
        };

        repo.create(&conflict).await.unwrap();
        repo.mark_resolved(conflict.id, user_id).await.unwrap();

        let retrieved = repo.get_by_id(conflict.id).await.unwrap();
        assert!(retrieved.resolved);
        assert!(retrieved.resolved_at.is_some());
        assert_eq!(retrieved.resolved_by, Some(user_id));
    }

    #[tokio::test]
    async fn test_list_unresolved() {
        let pool = setup_test_db().await;
        let repo = ConflictResolutionRepository::new(&pool);

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

        let conflict1 = ConflictResolution {
            id: Uuid::new_v4(),
            entity_type: ShareEntityType::Contact,
            entity_id: contact_id,
            local_version: 1,
            remote_version: 2,
            local_updated_at: Utc::now(),
            remote_updated_at: Utc::now(),
            local_changes: serde_json::json!({}),
            remote_changes: serde_json::json!({}),
            resolution_strategy: ResolutionStrategy::Manual,
            resolved: false,
            resolved_at: None,
            resolved_by: None,
            created_at: Utc::now(),
        };

        let conflict2 = ConflictResolution {
            id: Uuid::new_v4(),
            entity_type: ShareEntityType::Contact,
            entity_id: contact_id,
            local_version: 2,
            remote_version: 3,
            local_updated_at: Utc::now(),
            remote_updated_at: Utc::now(),
            local_changes: serde_json::json!({}),
            remote_changes: serde_json::json!({}),
            resolution_strategy: ResolutionStrategy::Manual,
            resolved: false,
            resolved_at: None,
            resolved_by: None,
            created_at: Utc::now(),
        };

        repo.create(&conflict1).await.unwrap();
        repo.create(&conflict2).await.unwrap();

        let unresolved = repo.list_unresolved(10, 0).await.unwrap();
        assert_eq!(unresolved.len(), 2);
    }
}
