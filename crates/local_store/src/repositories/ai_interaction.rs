use crate::db::DbPool;
use core_domain::{AiInteraction, DomainError, DomainResult};
use uuid::Uuid;

pub struct AiInteractionRepository<'a> {
    pool: &'a DbPool,
}

impl<'a> AiInteractionRepository<'a> {
    pub fn new(pool: &'a DbPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, interaction: &AiInteraction) -> DomainResult<()> {
        sqlx::query(
            "INSERT INTO ai_interactions (id, user_id, interaction_type, prompt, response, confidence, model, entity_type, entity_id, feedback_helpful, feedback_applied, metadata, created_at, feedback_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(interaction.id.to_string())
        .bind(interaction.user_id.to_string())
        .bind(&interaction.interaction_type)
        .bind(&interaction.prompt)
        .bind(&interaction.response)
        .bind(interaction.confidence)
        .bind(&interaction.model)
        .bind(&interaction.entity_type)
        .bind(interaction.entity_id.map(|id| id.to_string()))
        .bind(interaction.feedback_helpful.map(|h| if h { 1 } else { 0 }))
        .bind(interaction.feedback_applied as i32)
        .bind(serde_json::to_string(&interaction.metadata).unwrap())
        .bind(interaction.created_at.to_rfc3339())
        .bind(interaction.feedback_at.map(|dt| dt.to_rfc3339()))
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn get_by_id(&self, id: Uuid) -> DomainResult<AiInteraction> {
        let row = sqlx::query_as::<_, AiInteractionRow>(
            "SELECT id, user_id, interaction_type, prompt, response, confidence, model, entity_type, entity_id, feedback_helpful, feedback_applied, metadata, created_at, feedback_at
             FROM ai_interactions WHERE id = ?"
        )
        .bind(id.to_string())
        .fetch_optional(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?
        .ok_or_else(|| DomainError::NotFound(format!("AiInteraction {}", id)))?;

        Ok(row.into())
    }

    pub async fn list_by_user(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> DomainResult<Vec<AiInteraction>> {
        let rows = sqlx::query_as::<_, AiInteractionRow>(
            "SELECT id, user_id, interaction_type, prompt, response, confidence, model, entity_type, entity_id, feedback_helpful, feedback_applied, metadata, created_at, feedback_at
             FROM ai_interactions WHERE user_id = ? ORDER BY created_at DESC LIMIT ? OFFSET ?"
        )
        .bind(user_id.to_string())
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_by_entity(
        &self,
        entity_type: &str,
        entity_id: Uuid,
    ) -> DomainResult<Vec<AiInteraction>> {
        let rows = sqlx::query_as::<_, AiInteractionRow>(
            "SELECT id, user_id, interaction_type, prompt, response, confidence, model, entity_type, entity_id, feedback_helpful, feedback_applied, metadata, created_at, feedback_at
             FROM ai_interactions WHERE entity_type = ? AND entity_id = ? ORDER BY created_at DESC"
        )
        .bind(entity_type)
        .bind(entity_id.to_string())
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_feedback(
        &self,
        id: Uuid,
        helpful: bool,
        applied: bool,
    ) -> DomainResult<()> {
        let feedback_at = chrono::Utc::now();

        sqlx::query(
            "UPDATE ai_interactions SET feedback_helpful = ?, feedback_applied = ?, feedback_at = ? WHERE id = ?"
        )
        .bind(if helpful { 1 } else { 0 })
        .bind(applied as i32)
        .bind(feedback_at.to_rfc3339())
        .bind(id.to_string())
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn delete(&self, id: Uuid) -> DomainResult<()> {
        sqlx::query("DELETE FROM ai_interactions WHERE id = ?")
            .bind(id.to_string())
            .execute(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn list_recent(&self, limit: i64) -> DomainResult<Vec<AiInteraction>> {
        let rows = sqlx::query_as::<_, AiInteractionRow>(
            "SELECT id, user_id, interaction_type, prompt, response, confidence, model, entity_type, entity_id, feedback_helpful, feedback_applied, metadata, created_at, feedback_at
             FROM ai_interactions ORDER BY created_at DESC LIMIT ?"
        )
        .bind(limit)
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }
}

#[derive(sqlx::FromRow)]
struct AiInteractionRow {
    id: String,
    user_id: String,
    interaction_type: String,
    prompt: String,
    response: String,
    confidence: f32,
    model: String,
    entity_type: Option<String>,
    entity_id: Option<String>,
    feedback_helpful: Option<i32>,
    feedback_applied: i32,
    metadata: String,
    created_at: String,
    feedback_at: Option<String>,
}

impl From<AiInteractionRow> for AiInteraction {
    fn from(row: AiInteractionRow) -> Self {
        Self {
            id: Uuid::parse_str(&row.id).unwrap(),
            user_id: Uuid::parse_str(&row.user_id).unwrap(),
            interaction_type: row.interaction_type,
            prompt: row.prompt,
            response: row.response,
            confidence: row.confidence,
            model: row.model,
            entity_type: row.entity_type,
            entity_id: row.entity_id.and_then(|id| Uuid::parse_str(&id).ok()),
            feedback_helpful: row.feedback_helpful.map(|v| v != 0),
            feedback_applied: row.feedback_applied != 0,
            metadata: serde_json::from_str(&row.metadata).unwrap_or(serde_json::json!({})),
            created_at: chrono::DateTime::parse_from_rfc3339(&row.created_at)
                .unwrap()
                .with_timezone(&chrono::Utc),
            feedback_at: row.feedback_at.and_then(|dt| {
                chrono::DateTime::parse_from_rfc3339(&dt)
                    .ok()
                    .map(|dt| dt.with_timezone(&chrono::Utc))
            }),
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
    async fn test_create_and_get_interaction() {
        let pool = setup_test_db().await;
        let repo = AiInteractionRepository::new(&pool);

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

        let interaction = AiInteraction {
            id: Uuid::new_v4(),
            user_id,
            interaction_type: "TagSuggestion".to_string(),
            prompt: "Suggest tags for this contact".to_string(),
            response: "Professional, Tech Industry".to_string(),
            confidence: 0.85,
            model: "llama-3.1-8b".to_string(),
            entity_type: Some("Contact".to_string()),
            entity_id: Some(Uuid::new_v4()),
            feedback_helpful: None,
            feedback_applied: false,
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
            feedback_at: None,
        };

        repo.create(&interaction).await.unwrap();

        let retrieved = repo.get_by_id(interaction.id).await.unwrap();
        assert_eq!(retrieved.prompt, interaction.prompt);
        assert_eq!(retrieved.confidence, 0.85);
    }

    #[tokio::test]
    async fn test_update_feedback() {
        let pool = setup_test_db().await;
        let repo = AiInteractionRepository::new(&pool);

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

        let interaction = AiInteraction {
            id: Uuid::new_v4(),
            user_id,
            interaction_type: "NextAction".to_string(),
            prompt: "Suggest next action".to_string(),
            response: "Follow up within 2 weeks".to_string(),
            confidence: 0.9,
            model: "llama-3.1-8b".to_string(),
            entity_type: Some("Contact".to_string()),
            entity_id: Some(Uuid::new_v4()),
            feedback_helpful: None,
            feedback_applied: false,
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
            feedback_at: None,
        };

        repo.create(&interaction).await.unwrap();

        repo.update_feedback(interaction.id, true, true)
            .await
            .unwrap();

        let updated = repo.get_by_id(interaction.id).await.unwrap();
        assert_eq!(updated.feedback_helpful, Some(true));
        assert_eq!(updated.feedback_applied, true);
        assert!(updated.feedback_at.is_some());
    }
}

#[cfg(test)]
#[path = "ai_interaction_tests.rs"]
mod ai_interaction_tests;
