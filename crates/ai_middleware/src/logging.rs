use crate::segmind::{SegmindClient, SegmindResponse};
use anyhow::Result;
use chrono::Utc;
use sqlx::{Pool, Sqlite};
use tracing::{error, info};
use uuid::Uuid;

/// Parameters for logging an AI interaction, reducing argument count.
struct LogInteractionParams<'a> {
    user_id: Uuid,
    interaction_type: &'a str,
    prompt: &'a str,
    response: &'a str,
    confidence: f32,
    model: &'a str,
    entity_type: Option<String>,
    entity_id: Option<Uuid>,
}

/// Wrapper around SegmindClient that logs all interactions
pub struct LoggingSegmindClient {
    client: SegmindClient,
    pool: Pool<Sqlite>,
}

impl LoggingSegmindClient {
    pub fn new(client: SegmindClient, pool: Pool<Sqlite>) -> Self {
        Self { client, pool }
    }

    /// Generate AI suggestion with logging
    pub async fn generate_suggestion_with_logging(
        &self,
        prompt: &str,
        user_id: Uuid,
        entity_type: Option<String>,
        entity_id: Option<Uuid>,
        interaction_type: &str,
    ) -> Result<SegmindResponse> {
        // Call the underlying client
        let response = self.client.generate_suggestion(prompt).await?;

        // Log the interaction
        if let Err(e) = self
            .log_interaction(LogInteractionParams {
                user_id,
                interaction_type,
                prompt,
                response: &response.text,
                confidence: response.confidence,
                model: &response.model,
                entity_type,
                entity_id,
            })
            .await
        {
            error!("Failed to log AI interaction: {}", e);
            // Don't fail the whole operation if logging fails
        }

        Ok(response)
    }

    /// Analyze contact data with logging
    pub async fn analyze_contact_data_with_logging(
        &self,
        contact_data: &str,
        user_id: Uuid,
        contact_id: Uuid,
    ) -> Result<crate::segmind::ContactAnalysis> {
        // Call the underlying client
        let response = self.client.analyze_contact_data(contact_data).await?;

        // Log the interaction
        let response_text = format!(
            "Tags: {:?}, Strength: {}, Frequency: {}, Next: {}",
            response.suggested_tags,
            response.relationship_strength,
            response.communication_frequency,
            response.next_action
        );
        let prompt_text = format!("Analyze contact: {}", contact_data);
        if let Err(e) = self
            .log_interaction(LogInteractionParams {
                user_id,
                interaction_type: "ContactAnalysis",
                prompt: &prompt_text,
                response: &response_text,
                confidence: response.relationship_strength,
                model: "segmind-contact-analyzer",
                entity_type: Some("Contact".to_string()),
                entity_id: Some(contact_id),
            })
            .await
        {
            error!("Failed to log AI interaction: {}", e);
        }

        Ok(response)
    }

    /// Record feedback on an AI interaction
    pub async fn record_feedback(
        &self,
        interaction_id: Uuid,
        helpful: bool,
        applied: bool,
    ) -> Result<()> {
        let feedback_at = Utc::now();

        sqlx::query(
            "UPDATE ai_interactions SET feedback_helpful = ?, feedback_applied = ?, feedback_at = ? WHERE id = ?"
        )
        .bind(if helpful { 1 } else { 0 })
        .bind(applied as i32)
        .bind(feedback_at.to_rfc3339())
        .bind(interaction_id.to_string())
        .execute(&self.pool)
        .await?;

        info!(
            "Recorded feedback for interaction {}: helpful={}, applied={}",
            interaction_id, helpful, applied
        );

        Ok(())
    }

    /// Get recent AI interactions for a user
    pub async fn get_user_interactions(
        &self,
        user_id: Uuid,
        limit: i64,
    ) -> Result<Vec<AiInteractionRecord>> {
        let rows = sqlx::query_as::<_, AiInteractionRow>(
            "SELECT id, user_id, interaction_type, prompt, response, confidence, model, entity_type, entity_id, feedback_helpful, feedback_applied, metadata, created_at, feedback_at
             FROM ai_interactions WHERE user_id = ? ORDER BY created_at DESC LIMIT ?"
        )
        .bind(user_id.to_string())
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    /// Get AI interactions for a specific entity
    pub async fn get_entity_interactions(
        &self,
        entity_type: &str,
        entity_id: Uuid,
    ) -> Result<Vec<AiInteractionRecord>> {
        let rows = sqlx::query_as::<_, AiInteractionRow>(
            "SELECT id, user_id, interaction_type, prompt, response, confidence, model, entity_type, entity_id, feedback_helpful, feedback_applied, metadata, created_at, feedback_at
             FROM ai_interactions WHERE entity_type = ? AND entity_id = ? ORDER BY created_at DESC"
        )
        .bind(entity_type)
        .bind(entity_id.to_string())
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    /// Private helper to log an interaction
    async fn log_interaction(&self, params: LogInteractionParams<'_>) -> Result<Uuid> {
        let id = Uuid::new_v4();
        let created_at = Utc::now();

        sqlx::query(
            "INSERT INTO ai_interactions (id, user_id, interaction_type, prompt, response, confidence, model, entity_type, entity_id, feedback_helpful, feedback_applied, metadata, created_at, feedback_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(id.to_string())
        .bind(params.user_id.to_string())
        .bind(params.interaction_type)
        .bind(params.prompt)
        .bind(params.response)
        .bind(params.confidence)
        .bind(params.model)
        .bind(&params.entity_type)
        .bind(params.entity_id.map(|id| id.to_string()))
        .bind(None::<i32>) // feedback_helpful
        .bind(0) // feedback_applied
        .bind(serde_json::to_string(&serde_json::json!({}))?)
        .bind(created_at.to_rfc3339())
        .bind(None::<String>) // feedback_at
        .execute(&self.pool)
        .await?;

        info!(
            "Logged AI interaction {}: type={}, model={}",
            id, params.interaction_type, params.model
        );

        Ok(id)
    }
}

#[derive(Debug, Clone)]
pub struct AiInteractionRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub interaction_type: String,
    pub prompt: String,
    pub response: String,
    pub confidence: f32,
    pub model: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<Uuid>,
    pub feedback_helpful: Option<bool>,
    pub feedback_applied: bool,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<Utc>,
    pub feedback_at: Option<chrono::DateTime<Utc>>,
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

impl From<AiInteractionRow> for AiInteractionRecord {
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
                .with_timezone(&Utc),
            feedback_at: row.feedback_at.and_then(|dt| {
                chrono::DateTime::parse_from_rfc3339(&dt)
                    .ok()
                    .map(|dt| dt.with_timezone(&Utc))
            }),
        }
    }
}
