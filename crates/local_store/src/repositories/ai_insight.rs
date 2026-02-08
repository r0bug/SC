use crate::db::DbPool;
use core_domain::{
    AiInsight, AiInsightEntityType, AiInsightFeedback, AiInsightType, DomainError, DomainResult,
};
use uuid::Uuid;

pub struct AiInsightRepository<'a> {
    pool: &'a DbPool,
}

impl<'a> AiInsightRepository<'a> {
    pub fn new(pool: &'a DbPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, insight: &AiInsight) -> DomainResult<()> {
        let entity_type_str = match insight.entity_type {
            AiInsightEntityType::Contact => "Contact",
            AiInsightEntityType::Project => "Project",
            AiInsightEntityType::Note => "Note",
            AiInsightEntityType::Communication => "Communication",
            AiInsightEntityType::CalendarEvent => "CalendarEvent",
        };

        let insight_type_str = match insight.insight_type {
            AiInsightType::TagSuggestion => "TagSuggestion",
            AiInsightType::ChannelRecommendation => "ChannelRecommendation",
            AiInsightType::NextAction => "NextAction",
            AiInsightType::RelationshipStrength => "RelationshipStrength",
            AiInsightType::ContentSummary => "ContentSummary",
        };

        sqlx::query(
            "INSERT INTO ai_insights (id, entity_type, entity_id, insight_type, content, confidence, prompt_template, response_cached, feedback_helpful, feedback_comment, feedback_submitted_at, applied, applied_at, created_at, created_by)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(insight.id.to_string())
        .bind(entity_type_str)
        .bind(insight.entity_id.to_string())
        .bind(insight_type_str)
        .bind(&insight.content)
        .bind(insight.confidence)
        .bind(&insight.prompt_template)
        .bind(insight.response_cached as i32)
        .bind(insight.feedback.as_ref().map(|f| f.helpful as i32))
        .bind(insight.feedback.as_ref().and_then(|f| f.comment.clone()))
        .bind(insight.feedback.as_ref().map(|f| f.submitted_at.to_rfc3339()))
        .bind(insight.applied as i32)
        .bind(insight.applied_at.map(|t| t.to_rfc3339()))
        .bind(insight.created_at.to_rfc3339())
        .bind(insight.created_by.to_string())
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn get_by_id(&self, id: Uuid) -> DomainResult<AiInsight> {
        let row = sqlx::query_as::<_, AiInsightRow>(
            "SELECT id, entity_type, entity_id, insight_type, content, confidence, prompt_template, response_cached, feedback_helpful, feedback_comment, feedback_submitted_at, applied, applied_at, created_at, created_by
             FROM ai_insights WHERE id = ?"
        )
        .bind(id.to_string())
        .fetch_optional(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?
        .ok_or_else(|| DomainError::NotFound(format!("AiInsight {}", id)))?;

        Ok(row.into())
    }

    pub async fn list_by_entity(
        &self,
        entity_type: AiInsightEntityType,
        entity_id: Uuid,
    ) -> DomainResult<Vec<AiInsight>> {
        let entity_type_str = match entity_type {
            AiInsightEntityType::Contact => "Contact",
            AiInsightEntityType::Project => "Project",
            AiInsightEntityType::Note => "Note",
            AiInsightEntityType::Communication => "Communication",
            AiInsightEntityType::CalendarEvent => "CalendarEvent",
        };

        let rows = sqlx::query_as::<_, AiInsightRow>(
            "SELECT id, entity_type, entity_id, insight_type, content, confidence, prompt_template, response_cached, feedback_helpful, feedback_comment, feedback_submitted_at, applied, applied_at, created_at, created_by
             FROM ai_insights WHERE entity_type = ? AND entity_id = ? ORDER BY created_at DESC"
        )
        .bind(entity_type_str)
        .bind(entity_id.to_string())
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn submit_feedback(
        &self,
        id: Uuid,
        feedback: &AiInsightFeedback,
    ) -> DomainResult<()> {
        sqlx::query(
            "UPDATE ai_insights SET feedback_helpful = ?, feedback_comment = ?, feedback_submitted_at = ?
             WHERE id = ?"
        )
        .bind(feedback.helpful as i32)
        .bind(&feedback.comment)
        .bind(feedback.submitted_at.to_rfc3339())
        .bind(id.to_string())
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn mark_applied(&self, id: Uuid) -> DomainResult<()> {
        sqlx::query("UPDATE ai_insights SET applied = 1, applied_at = ? WHERE id = ?")
            .bind(chrono::Utc::now().to_rfc3339())
            .bind(id.to_string())
            .execute(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn delete(&self, id: Uuid) -> DomainResult<()> {
        sqlx::query("DELETE FROM ai_insights WHERE id = ?")
            .bind(id.to_string())
            .execute(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct AiInsightRow {
    id: String,
    entity_type: String,
    entity_id: String,
    insight_type: String,
    content: String,
    confidence: f32,
    prompt_template: String,
    response_cached: i32,
    feedback_helpful: Option<i32>,
    feedback_comment: Option<String>,
    feedback_submitted_at: Option<String>,
    applied: i32,
    applied_at: Option<String>,
    created_at: String,
    created_by: String,
}

impl From<AiInsightRow> for AiInsight {
    fn from(row: AiInsightRow) -> Self {
        let entity_type = match row.entity_type.as_str() {
            "Contact" => AiInsightEntityType::Contact,
            "Project" => AiInsightEntityType::Project,
            "Note" => AiInsightEntityType::Note,
            "Communication" => AiInsightEntityType::Communication,
            "CalendarEvent" => AiInsightEntityType::CalendarEvent,
            _ => AiInsightEntityType::Contact,
        };

        let insight_type = match row.insight_type.as_str() {
            "TagSuggestion" => AiInsightType::TagSuggestion,
            "ChannelRecommendation" => AiInsightType::ChannelRecommendation,
            "NextAction" => AiInsightType::NextAction,
            "RelationshipStrength" => AiInsightType::RelationshipStrength,
            "ContentSummary" => AiInsightType::ContentSummary,
            _ => AiInsightType::TagSuggestion,
        };

        let feedback = if let (Some(helpful), Some(submitted_at)) =
            (row.feedback_helpful, row.feedback_submitted_at.clone())
        {
            Some(AiInsightFeedback {
                helpful: helpful != 0,
                comment: row.feedback_comment.clone(),
                submitted_at: chrono::DateTime::parse_from_rfc3339(&submitted_at)
                    .unwrap()
                    .with_timezone(&chrono::Utc),
            })
        } else {
            None
        };

        Self {
            id: Uuid::parse_str(&row.id).unwrap(),
            entity_type,
            entity_id: Uuid::parse_str(&row.entity_id).unwrap(),
            insight_type,
            content: row.content,
            confidence: row.confidence,
            prompt_template: row.prompt_template,
            response_cached: row.response_cached != 0,
            feedback,
            applied: row.applied != 0,
            applied_at: row.applied_at.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|dt| dt.with_timezone(&chrono::Utc))
            }),
            created_at: chrono::DateTime::parse_from_rfc3339(&row.created_at)
                .unwrap()
                .with_timezone(&chrono::Utc),
            created_by: Uuid::parse_str(&row.created_by).unwrap(),
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
    async fn test_create_and_get_insight() {
        let pool = setup_test_db().await;
        let repo = AiInsightRepository::new(&pool);

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

        let insight = AiInsight {
            id: Uuid::new_v4(),
            entity_type: AiInsightEntityType::Contact,
            entity_id: contact_id,
            insight_type: AiInsightType::TagSuggestion,
            content: "Consider tagging this contact as VIP".to_string(),
            confidence: 0.85,
            prompt_template: "analyze_contact".to_string(),
            response_cached: false,
            feedback: None,
            applied: false,
            applied_at: None,
            created_at: Utc::now(),
            created_by: user_id,
        };

        repo.create(&insight).await.unwrap();

        let retrieved = repo.get_by_id(insight.id).await.unwrap();
        assert_eq!(retrieved.content, insight.content);
        assert_eq!(retrieved.confidence, 0.85);
    }

    #[tokio::test]
    async fn test_submit_feedback_and_mark_applied() {
        let pool = setup_test_db().await;
        let repo = AiInsightRepository::new(&pool);

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

        let insight = AiInsight {
            id: Uuid::new_v4(),
            entity_type: AiInsightEntityType::Contact,
            entity_id: contact_id,
            insight_type: AiInsightType::NextAction,
            content: "Follow up next week".to_string(),
            confidence: 0.75,
            prompt_template: "suggest_action".to_string(),
            response_cached: false,
            feedback: None,
            applied: false,
            applied_at: None,
            created_at: Utc::now(),
            created_by: user_id,
        };

        repo.create(&insight).await.unwrap();

        let feedback = AiInsightFeedback {
            helpful: true,
            comment: Some("Very helpful!".to_string()),
            submitted_at: Utc::now(),
        };

        repo.submit_feedback(insight.id, &feedback).await.unwrap();
        repo.mark_applied(insight.id).await.unwrap();

        let retrieved = repo.get_by_id(insight.id).await.unwrap();
        assert!(retrieved.feedback.is_some());
        assert!(retrieved.applied);
        assert!(retrieved.applied_at.is_some());
    }
}
