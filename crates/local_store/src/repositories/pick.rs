use core_domain::{DomainError, DomainResult, Pick, PickStatus};
use crate::db::DbPool;
use uuid::Uuid;

pub struct PickRepository<'a> {
    pool: &'a DbPool,
}

impl<'a> PickRepository<'a> {
    pub fn new(pool: &'a DbPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, pick: &Pick) -> DomainResult<()> {
        sqlx::query(
            "INSERT INTO picks (id, name, description, status, date_start, date_end, recurrence, metadata, created_at, updated_at, created_by)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(pick.id.to_string())
        .bind(&pick.name)
        .bind(&pick.description)
        .bind(pick.status.to_string())
        .bind(pick.date_start.map(|dt| dt.to_rfc3339()))
        .bind(pick.date_end.map(|dt| dt.to_rfc3339()))
        .bind(pick.recurrence.as_ref().map(|r| serde_json::to_string(r).unwrap()))
        .bind(serde_json::to_string(&pick.metadata).unwrap())
        .bind(pick.created_at.to_rfc3339())
        .bind(pick.updated_at.to_rfc3339())
        .bind(pick.created_by.to_string())
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(format!("Failed to insert pick: {}", e)))?;

        Ok(())
    }

    pub async fn get_by_id(&self, id: Uuid) -> DomainResult<Pick> {
        let row = sqlx::query_as::<_, PickRow>(
            "SELECT id, name, description, status, date_start, date_end, recurrence, metadata, created_at, updated_at, created_by
             FROM picks WHERE id = ?",
        )
        .bind(id.to_string())
        .fetch_optional(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?
        .ok_or_else(|| DomainError::NotFound(format!("Pick {}", id)))?;

        Ok(row.into_pick())
    }

    pub async fn list(&self, limit: i64, offset: i64) -> DomainResult<Vec<Pick>> {
        let rows = sqlx::query_as::<_, PickRow>(
            "SELECT id, name, description, status, date_start, date_end, recurrence, metadata, created_at, updated_at, created_by
             FROM picks ORDER BY name LIMIT ? OFFSET ?",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into_pick()).collect())
    }

    pub async fn list_by_creator(&self, creator_id: Uuid) -> DomainResult<Vec<Pick>> {
        let rows = sqlx::query_as::<_, PickRow>(
            "SELECT id, name, description, status, date_start, date_end, recurrence, metadata, created_at, updated_at, created_by
             FROM picks WHERE created_by = ? ORDER BY name",
        )
        .bind(creator_id.to_string())
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into_pick()).collect())
    }

    pub async fn list_by_status(&self, status: &PickStatus) -> DomainResult<Vec<Pick>> {
        let rows = sqlx::query_as::<_, PickRow>(
            "SELECT id, name, description, status, date_start, date_end, recurrence, metadata, created_at, updated_at, created_by
             FROM picks WHERE status = ? ORDER BY name",
        )
        .bind(status.to_string())
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into_pick()).collect())
    }

    pub async fn update(&self, pick: &Pick) -> DomainResult<()> {
        sqlx::query(
            "UPDATE picks SET name = ?, description = ?, status = ?, date_start = ?, date_end = ?, recurrence = ?, metadata = ?, updated_at = ?
             WHERE id = ?",
        )
        .bind(&pick.name)
        .bind(&pick.description)
        .bind(pick.status.to_string())
        .bind(pick.date_start.map(|dt| dt.to_rfc3339()))
        .bind(pick.date_end.map(|dt| dt.to_rfc3339()))
        .bind(pick.recurrence.as_ref().map(|r| serde_json::to_string(r).unwrap()))
        .bind(serde_json::to_string(&pick.metadata).unwrap())
        .bind(pick.updated_at.to_rfc3339())
        .bind(pick.id.to_string())
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn delete(&self, id: Uuid) -> DomainResult<()> {
        sqlx::query("DELETE FROM picks WHERE id = ?")
            .bind(id.to_string())
            .execute(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn search(&self, query: &str) -> DomainResult<Vec<Pick>> {
        let search_pattern = format!("%{}%", query);
        let rows = sqlx::query_as::<_, PickRow>(
            "SELECT id, name, description, status, date_start, date_end, recurrence, metadata, created_at, updated_at, created_by
             FROM picks
             WHERE name LIKE ? OR description LIKE ?
             ORDER BY name
             LIMIT 50",
        )
        .bind(&search_pattern)
        .bind(&search_pattern)
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into_pick()).collect())
    }
}

#[derive(sqlx::FromRow)]
struct PickRow {
    id: String,
    name: String,
    description: Option<String>,
    status: String,
    date_start: Option<String>,
    date_end: Option<String>,
    recurrence: Option<String>,
    metadata: String,
    created_at: String,
    updated_at: String,
    created_by: String,
}

impl PickRow {
    fn into_pick(self) -> Pick {
        let status = self
            .status
            .parse::<PickStatus>()
            .unwrap_or(PickStatus::Upcoming);

        Pick {
            id: Uuid::parse_str(&self.id).unwrap(),
            name: self.name,
            description: self.description,
            status,
            date_start: self.date_start.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|dt| dt.with_timezone(&chrono::Utc))
            }),
            date_end: self.date_end.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|dt| dt.with_timezone(&chrono::Utc))
            }),
            recurrence: self
                .recurrence
                .and_then(|s| serde_json::from_str(&s).ok()),
            metadata: serde_json::from_str(&self.metadata).unwrap_or_default(),
            created_at: chrono::DateTime::parse_from_rfc3339(&self.created_at)
                .unwrap()
                .with_timezone(&chrono::Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(&self.updated_at)
                .unwrap()
                .with_timezone(&chrono::Utc),
            created_by: Uuid::parse_str(&self.created_by).unwrap(),
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

    async fn insert_test_user(pool: &DbPool) -> Uuid {
        let user_id = Uuid::new_v4();
        sqlx::query("INSERT INTO users (id, email, name, password_hash, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(user_id.to_string())
            .bind("test@example.com")
            .bind("Test")
            .bind("hash")
            .bind(Utc::now().to_rfc3339())
            .bind(Utc::now().to_rfc3339())
            .execute(pool)
            .await
            .unwrap();
        user_id
    }

    fn make_pick(user_id: Uuid) -> Pick {
        Pick {
            id: Uuid::new_v4(),
            name: "Estate Sale - Downtown".to_string(),
            description: Some("Large estate sale with antiques".to_string()),
            status: PickStatus::Upcoming,
            date_start: Some(Utc::now()),
            date_end: Some(Utc::now() + chrono::Duration::hours(4)),
            recurrence: None,
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: user_id,
        }
    }

    #[tokio::test]
    async fn test_create_and_get_pick() {
        let pool = setup_test_db().await;
        let repo = PickRepository::new(&pool);
        let user_id = insert_test_user(&pool).await;

        let pick = make_pick(user_id);
        repo.create(&pick).await.unwrap();

        let retrieved = repo.get_by_id(pick.id).await.unwrap();
        assert_eq!(retrieved.name, pick.name);
        assert_eq!(retrieved.description, pick.description);
        assert_eq!(retrieved.status, PickStatus::Upcoming);
        assert_eq!(retrieved.created_by, user_id);
    }

    #[tokio::test]
    async fn test_list_picks() {
        let pool = setup_test_db().await;
        let repo = PickRepository::new(&pool);
        let user_id = insert_test_user(&pool).await;

        let pick1 = make_pick(user_id);
        let mut pick2 = make_pick(user_id);
        pick2.name = "Flea Market Saturday".to_string();

        repo.create(&pick1).await.unwrap();
        repo.create(&pick2).await.unwrap();

        let picks = repo.list(10, 0).await.unwrap();
        assert_eq!(picks.len(), 2);
    }

    #[tokio::test]
    async fn test_list_by_creator() {
        let pool = setup_test_db().await;
        let repo = PickRepository::new(&pool);
        let user_id = insert_test_user(&pool).await;

        let pick = make_pick(user_id);
        repo.create(&pick).await.unwrap();

        let picks = repo.list_by_creator(user_id).await.unwrap();
        assert_eq!(picks.len(), 1);
        assert_eq!(picks[0].name, pick.name);

        let other_id = Uuid::new_v4();
        let picks = repo.list_by_creator(other_id).await.unwrap();
        assert_eq!(picks.len(), 0);
    }

    #[tokio::test]
    async fn test_list_by_status() {
        let pool = setup_test_db().await;
        let repo = PickRepository::new(&pool);
        let user_id = insert_test_user(&pool).await;

        let mut pick1 = make_pick(user_id);
        pick1.status = PickStatus::Active;

        let pick2 = make_pick(user_id);

        repo.create(&pick1).await.unwrap();
        repo.create(&pick2).await.unwrap();

        let active = repo.list_by_status(&PickStatus::Active).await.unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].name, pick1.name);

        let upcoming = repo.list_by_status(&PickStatus::Upcoming).await.unwrap();
        assert_eq!(upcoming.len(), 1);
    }

    #[tokio::test]
    async fn test_update_pick() {
        let pool = setup_test_db().await;
        let repo = PickRepository::new(&pool);
        let user_id = insert_test_user(&pool).await;

        let mut pick = make_pick(user_id);
        repo.create(&pick).await.unwrap();

        pick.name = "Updated Estate Sale".to_string();
        pick.status = PickStatus::Active;
        pick.updated_at = Utc::now();
        repo.update(&pick).await.unwrap();

        let retrieved = repo.get_by_id(pick.id).await.unwrap();
        assert_eq!(retrieved.name, "Updated Estate Sale");
        assert_eq!(retrieved.status, PickStatus::Active);
    }

    #[tokio::test]
    async fn test_delete_pick() {
        let pool = setup_test_db().await;
        let repo = PickRepository::new(&pool);
        let user_id = insert_test_user(&pool).await;

        let pick = make_pick(user_id);
        repo.create(&pick).await.unwrap();

        repo.delete(pick.id).await.unwrap();

        let result = repo.get_by_id(pick.id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_search_picks() {
        let pool = setup_test_db().await;
        let repo = PickRepository::new(&pool);
        let user_id = insert_test_user(&pool).await;

        let pick1 = make_pick(user_id);
        let mut pick2 = make_pick(user_id);
        pick2.name = "Garage Sale Uptown".to_string();
        pick2.description = Some("Small garage sale".to_string());

        repo.create(&pick1).await.unwrap();
        repo.create(&pick2).await.unwrap();

        let results = repo.search("Estate").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, pick1.name);

        let results = repo.search("sale").await.unwrap();
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_get_nonexistent_pick() {
        let pool = setup_test_db().await;
        let repo = PickRepository::new(&pool);

        let result = repo.get_by_id(Uuid::new_v4()).await;
        assert!(result.is_err());
    }
}
