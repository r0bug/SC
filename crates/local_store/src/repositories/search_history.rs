use crate::db::DbPool;
use core_domain::{DomainError, DomainResult, SearchHistory};
use uuid::Uuid;

pub struct SearchHistoryRepository<'a> {
    pool: &'a DbPool,
}

impl<'a> SearchHistoryRepository<'a> {
    pub fn new(pool: &'a DbPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, history: &SearchHistory) -> DomainResult<()> {
        let result_ids_json = serde_json::to_string(
            &history
                .result_ids
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>(),
        )
        .unwrap();

        sqlx::query(
            "INSERT INTO search_history (id, user_id, query, filters, result_count, result_ids, clicked_result_id, privacy_mode, metadata, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(history.id.to_string())
        .bind(history.user_id.to_string())
        .bind(&history.query)
        .bind(serde_json::to_string(&history.filters).unwrap())
        .bind(history.result_count)
        .bind(result_ids_json)
        .bind(history.clicked_result_id.map(|id| id.to_string()))
        .bind(history.privacy_mode as i32)
        .bind(serde_json::to_string(&history.metadata).unwrap())
        .bind(history.created_at.to_rfc3339())
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn get_by_id(&self, id: Uuid) -> DomainResult<SearchHistory> {
        let row = sqlx::query_as::<_, SearchHistoryRow>(
            "SELECT id, user_id, query, filters, result_count, result_ids, clicked_result_id, privacy_mode, metadata, created_at
             FROM search_history WHERE id = ?"
        )
        .bind(id.to_string())
        .fetch_optional(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?
        .ok_or_else(|| DomainError::NotFound(format!("SearchHistory {}", id)))?;

        Ok(row.into())
    }

    pub async fn list_by_user(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> DomainResult<Vec<SearchHistory>> {
        let rows = sqlx::query_as::<_, SearchHistoryRow>(
            "SELECT id, user_id, query, filters, result_count, result_ids, clicked_result_id, privacy_mode, metadata, created_at
             FROM search_history WHERE user_id = ? ORDER BY created_at DESC LIMIT ? OFFSET ?"
        )
        .bind(user_id.to_string())
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_clicked_result(
        &self,
        id: Uuid,
        clicked_result_id: Uuid,
    ) -> DomainResult<()> {
        sqlx::query("UPDATE search_history SET clicked_result_id = ? WHERE id = ?")
            .bind(clicked_result_id.to_string())
            .bind(id.to_string())
            .execute(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn delete(&self, id: Uuid) -> DomainResult<()> {
        sqlx::query("DELETE FROM search_history WHERE id = ?")
            .bind(id.to_string())
            .execute(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn delete_by_user(&self, user_id: Uuid) -> DomainResult<()> {
        sqlx::query("DELETE FROM search_history WHERE user_id = ?")
            .bind(user_id.to_string())
            .execute(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn clear_user_history(&self, user_id: Uuid) -> DomainResult<()> {
        self.delete_by_user(user_id).await
    }
}

#[derive(sqlx::FromRow)]
struct SearchHistoryRow {
    id: String,
    user_id: String,
    query: String,
    filters: String,
    result_count: i32,
    result_ids: String,
    clicked_result_id: Option<String>,
    privacy_mode: i32,
    metadata: String,
    created_at: String,
}

impl From<SearchHistoryRow> for SearchHistory {
    fn from(row: SearchHistoryRow) -> Self {
        let result_ids: Vec<String> = serde_json::from_str(&row.result_ids).unwrap_or_default();
        let result_ids: Vec<Uuid> = result_ids
            .iter()
            .filter_map(|s| Uuid::parse_str(s).ok())
            .collect();

        Self {
            id: Uuid::parse_str(&row.id).unwrap(),
            user_id: Uuid::parse_str(&row.user_id).unwrap(),
            query: row.query,
            filters: serde_json::from_str(&row.filters).unwrap_or(serde_json::json!({})),
            result_count: row.result_count,
            result_ids,
            clicked_result_id: row.clicked_result_id.and_then(|s| Uuid::parse_str(&s).ok()),
            privacy_mode: row.privacy_mode != 0,
            metadata: serde_json::from_str(&row.metadata).unwrap_or(serde_json::json!({})),
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
    async fn test_create_and_get_search_history() {
        let pool = setup_test_db().await;
        let repo = SearchHistoryRepository::new(&pool);

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

        let history = SearchHistory {
            id: Uuid::new_v4(),
            user_id,
            query: "john doe".to_string(),
            filters: serde_json::json!({"tags": ["important"]}),
            result_count: 5,
            result_ids: vec![Uuid::new_v4(), Uuid::new_v4()],
            clicked_result_id: None,
            privacy_mode: false,
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
        };

        repo.create(&history).await.unwrap();

        let retrieved = repo.get_by_id(history.id).await.unwrap();
        assert_eq!(retrieved.query, history.query);
        assert_eq!(retrieved.result_count, 5);
    }

    #[tokio::test]
    async fn test_list_by_user() {
        let pool = setup_test_db().await;
        let repo = SearchHistoryRepository::new(&pool);

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

        let history1 = SearchHistory {
            id: Uuid::new_v4(),
            user_id,
            query: "query1".to_string(),
            filters: serde_json::json!({}),
            result_count: 3,
            result_ids: vec![],
            clicked_result_id: None,
            privacy_mode: false,
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
        };

        let history2 = SearchHistory {
            id: Uuid::new_v4(),
            user_id,
            query: "query2".to_string(),
            filters: serde_json::json!({}),
            result_count: 7,
            result_ids: vec![],
            clicked_result_id: None,
            privacy_mode: false,
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
        };

        repo.create(&history1).await.unwrap();
        repo.create(&history2).await.unwrap();

        let results = repo.list_by_user(user_id, 10, 0).await.unwrap();
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_update_clicked_result() {
        let pool = setup_test_db().await;
        let repo = SearchHistoryRepository::new(&pool);

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

        let history = SearchHistory {
            id: Uuid::new_v4(),
            user_id,
            query: "test".to_string(),
            filters: serde_json::json!({}),
            result_count: 1,
            result_ids: vec![],
            clicked_result_id: None,
            privacy_mode: false,
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
        };

        repo.create(&history).await.unwrap();

        let clicked_id = Uuid::new_v4();
        repo.update_clicked_result(history.id, clicked_id)
            .await
            .unwrap();

        let retrieved = repo.get_by_id(history.id).await.unwrap();
        assert_eq!(retrieved.clicked_result_id, Some(clicked_id));
    }
}
