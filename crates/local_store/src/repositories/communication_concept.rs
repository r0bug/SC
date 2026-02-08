use core_domain::{CommunicationConcept, DetectionMethod, DomainError, DomainResult, SuggestionStatus};
use crate::db::DbPool;
use uuid::Uuid;

pub struct CommunicationConceptRepository<'a> {
    pool: &'a DbPool,
}

impl<'a> CommunicationConceptRepository<'a> {
    pub fn new(pool: &'a DbPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, cc: &CommunicationConcept) -> DomainResult<()> {
        sqlx::query(
            "INSERT INTO communication_concepts (id, communication_type, communication_id, concept_id, detection_method, status, confidence, reviewed_at, reviewed_by, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(cc.id.to_string())
        .bind(&cc.communication_type)
        .bind(cc.communication_id.to_string())
        .bind(cc.concept_id.to_string())
        .bind(cc.detection_method.to_string())
        .bind(cc.status.to_string())
        .bind(cc.confidence.map(|c| c as f64))
        .bind(cc.reviewed_at.map(|t| t.to_rfc3339()))
        .bind(cc.reviewed_by.map(|u| u.to_string()))
        .bind(cc.created_at.to_rfc3339())
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn get_by_id(&self, id: Uuid) -> DomainResult<CommunicationConcept> {
        let row = sqlx::query_as::<_, CommunicationConceptRow>(
            "SELECT id, communication_type, communication_id, concept_id, detection_method, status, confidence, reviewed_at, reviewed_by, created_at
             FROM communication_concepts WHERE id = ?",
        )
        .bind(id.to_string())
        .fetch_optional(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?
        .ok_or_else(|| DomainError::NotFound(format!("CommunicationConcept {}", id)))?;

        Ok(row.into_communication_concept())
    }

    pub async fn list_by_communication(
        &self,
        communication_type: &str,
        communication_id: Uuid,
    ) -> DomainResult<Vec<CommunicationConcept>> {
        let rows = sqlx::query_as::<_, CommunicationConceptRow>(
            "SELECT id, communication_type, communication_id, concept_id, detection_method, status, confidence, reviewed_at, reviewed_by, created_at
             FROM communication_concepts
             WHERE communication_type = ? AND communication_id = ?
             ORDER BY created_at DESC",
        )
        .bind(communication_type)
        .bind(communication_id.to_string())
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| r.into_communication_concept())
            .collect())
    }

    pub async fn list_by_concept(
        &self,
        concept_id: Uuid,
    ) -> DomainResult<Vec<CommunicationConcept>> {
        let rows = sqlx::query_as::<_, CommunicationConceptRow>(
            "SELECT id, communication_type, communication_id, concept_id, detection_method, status, confidence, reviewed_at, reviewed_by, created_at
             FROM communication_concepts
             WHERE concept_id = ?
             ORDER BY created_at DESC",
        )
        .bind(concept_id.to_string())
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| r.into_communication_concept())
            .collect())
    }

    pub async fn list_pending(&self) -> DomainResult<Vec<CommunicationConcept>> {
        let rows = sqlx::query_as::<_, CommunicationConceptRow>(
            "SELECT id, communication_type, communication_id, concept_id, detection_method, status, confidence, reviewed_at, reviewed_by, created_at
             FROM communication_concepts
             WHERE status = 'suggested'
             ORDER BY created_at DESC",
        )
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| r.into_communication_concept())
            .collect())
    }

    pub async fn update_status(
        &self,
        id: Uuid,
        status: &SuggestionStatus,
        reviewer_id: Uuid,
    ) -> DomainResult<()> {
        sqlx::query(
            "UPDATE communication_concepts SET status = ?, reviewed_at = ?, reviewed_by = ? WHERE id = ?",
        )
        .bind(status.to_string())
        .bind(chrono::Utc::now().to_rfc3339())
        .bind(reviewer_id.to_string())
        .bind(id.to_string())
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn delete(&self, id: Uuid) -> DomainResult<()> {
        sqlx::query("DELETE FROM communication_concepts WHERE id = ?")
            .bind(id.to_string())
            .execute(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    /// Delete link between a specific communication and concept
    pub async fn delete_by_communication_and_concept(
        &self,
        communication_type: &str,
        communication_id: Uuid,
        concept_id: Uuid,
    ) -> DomainResult<()> {
        sqlx::query(
            "DELETE FROM communication_concepts
             WHERE communication_type = ? AND communication_id = ? AND concept_id = ?",
        )
        .bind(communication_type)
        .bind(communication_id.to_string())
        .bind(concept_id.to_string())
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct CommunicationConceptRow {
    id: String,
    communication_type: String,
    communication_id: String,
    concept_id: String,
    detection_method: String,
    status: String,
    confidence: Option<f64>,
    reviewed_at: Option<String>,
    reviewed_by: Option<String>,
    created_at: String,
}

impl CommunicationConceptRow {
    fn into_communication_concept(self) -> CommunicationConcept {
        CommunicationConcept {
            id: Uuid::parse_str(&self.id).unwrap(),
            communication_type: self.communication_type,
            communication_id: Uuid::parse_str(&self.communication_id).unwrap(),
            concept_id: Uuid::parse_str(&self.concept_id).unwrap(),
            detection_method: self.detection_method.parse::<DetectionMethod>().unwrap(),
            status: self.status.parse::<SuggestionStatus>().unwrap(),
            confidence: self.confidence.map(|c| c as f32),
            reviewed_at: self.reviewed_at.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|dt| dt.with_timezone(&chrono::Utc))
            }),
            reviewed_by: self
                .reviewed_by
                .and_then(|s| Uuid::parse_str(&s).ok()),
            created_at: chrono::DateTime::parse_from_rfc3339(&self.created_at)
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

    async fn insert_test_concept(pool: &DbPool, user_id: Uuid) -> Uuid {
        let concept_id = Uuid::new_v4();
        sqlx::query("INSERT INTO concepts (id, name, created_at, updated_at, created_by) VALUES (?, ?, ?, ?, ?)")
            .bind(concept_id.to_string())
            .bind("Test Concept")
            .bind(Utc::now().to_rfc3339())
            .bind(Utc::now().to_rfc3339())
            .bind(user_id.to_string())
            .execute(pool)
            .await
            .unwrap();
        concept_id
    }

    fn make_communication_concept(concept_id: Uuid) -> CommunicationConcept {
        CommunicationConcept {
            id: Uuid::new_v4(),
            communication_type: "sms".to_string(),
            communication_id: Uuid::new_v4(),
            concept_id,
            detection_method: DetectionMethod::Keyword,
            status: SuggestionStatus::Suggested,
            confidence: Some(0.85),
            reviewed_at: None,
            reviewed_by: None,
            created_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_create_and_get() {
        let pool = setup_test_db().await;
        let repo = CommunicationConceptRepository::new(&pool);
        let user_id = insert_test_user(&pool).await;
        let concept_id = insert_test_concept(&pool, user_id).await;

        let cc = make_communication_concept(concept_id);
        repo.create(&cc).await.unwrap();

        let retrieved = repo.get_by_id(cc.id).await.unwrap();
        assert_eq!(retrieved.id, cc.id);
        assert_eq!(retrieved.communication_type, "sms");
        assert_eq!(retrieved.concept_id, concept_id);
        assert_eq!(retrieved.detection_method, DetectionMethod::Keyword);
        assert_eq!(retrieved.status, SuggestionStatus::Suggested);
        // f32 precision: compare with tolerance
        assert!((retrieved.confidence.unwrap() - 0.85).abs() < 0.001);
        assert!(retrieved.reviewed_at.is_none());
        assert!(retrieved.reviewed_by.is_none());
    }

    #[tokio::test]
    async fn test_list_by_communication() {
        let pool = setup_test_db().await;
        let repo = CommunicationConceptRepository::new(&pool);
        let user_id = insert_test_user(&pool).await;
        let concept_id = insert_test_concept(&pool, user_id).await;

        let comm_id = Uuid::new_v4();
        let mut cc1 = make_communication_concept(concept_id);
        cc1.communication_id = comm_id;
        cc1.communication_type = "email".to_string();

        let mut cc2 = make_communication_concept(concept_id);
        cc2.communication_id = comm_id;
        cc2.communication_type = "email".to_string();

        let cc3 = make_communication_concept(concept_id); // different comm_id

        repo.create(&cc1).await.unwrap();
        repo.create(&cc2).await.unwrap();
        repo.create(&cc3).await.unwrap();

        let results = repo.list_by_communication("email", comm_id).await.unwrap();
        assert_eq!(results.len(), 2);

        let results = repo.list_by_communication("sms", cc3.communication_id).await.unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_list_by_concept() {
        let pool = setup_test_db().await;
        let repo = CommunicationConceptRepository::new(&pool);
        let user_id = insert_test_user(&pool).await;
        let concept_id = insert_test_concept(&pool, user_id).await;

        let cc1 = make_communication_concept(concept_id);
        let cc2 = make_communication_concept(concept_id);

        repo.create(&cc1).await.unwrap();
        repo.create(&cc2).await.unwrap();

        let results = repo.list_by_concept(concept_id).await.unwrap();
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_list_pending() {
        let pool = setup_test_db().await;
        let repo = CommunicationConceptRepository::new(&pool);
        let user_id = insert_test_user(&pool).await;
        let concept_id = insert_test_concept(&pool, user_id).await;

        let cc1 = make_communication_concept(concept_id);
        let mut cc2 = make_communication_concept(concept_id);
        cc2.status = SuggestionStatus::Confirmed;

        repo.create(&cc1).await.unwrap();
        repo.create(&cc2).await.unwrap();

        let pending = repo.list_pending().await.unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].id, cc1.id);
    }

    #[tokio::test]
    async fn test_update_status() {
        let pool = setup_test_db().await;
        let repo = CommunicationConceptRepository::new(&pool);
        let user_id = insert_test_user(&pool).await;
        let concept_id = insert_test_concept(&pool, user_id).await;

        let cc = make_communication_concept(concept_id);
        repo.create(&cc).await.unwrap();

        repo.update_status(cc.id, &SuggestionStatus::Confirmed, user_id)
            .await
            .unwrap();

        let retrieved = repo.get_by_id(cc.id).await.unwrap();
        assert_eq!(retrieved.status, SuggestionStatus::Confirmed);
        assert!(retrieved.reviewed_at.is_some());
        assert_eq!(retrieved.reviewed_by, Some(user_id));
    }

    #[tokio::test]
    async fn test_delete() {
        let pool = setup_test_db().await;
        let repo = CommunicationConceptRepository::new(&pool);
        let user_id = insert_test_user(&pool).await;
        let concept_id = insert_test_concept(&pool, user_id).await;

        let cc = make_communication_concept(concept_id);
        repo.create(&cc).await.unwrap();

        repo.delete(cc.id).await.unwrap();

        let result = repo.get_by_id(cc.id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_not_found() {
        let pool = setup_test_db().await;
        let repo = CommunicationConceptRepository::new(&pool);

        let result = repo.get_by_id(Uuid::new_v4()).await;
        assert!(matches!(result, Err(DomainError::NotFound(_))));
    }
}
