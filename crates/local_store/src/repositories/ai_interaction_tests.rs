#[cfg(test)]
mod ai_interaction_integration_tests {
    use super::super::*;
    use crate::repositories::ai_interaction::AiInteractionRepository;
    use core_domain::AiInteraction;
    use chrono::Utc;
    use sqlx::sqlite::SqlitePoolOptions;
    use sqlx::{Pool, Sqlite};
    use uuid::Uuid;

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

    async fn create_test_user(pool: &Pool<Sqlite>) -> Uuid {
        let user_id = Uuid::new_v4();
        sqlx::query("INSERT INTO users (id, email, name, password_hash, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(user_id.to_string())
            .bind("test@example.com")
            .bind("Test User")
            .bind("hash")
            .bind(Utc::now().to_rfc3339())
            .bind(Utc::now().to_rfc3339())
            .execute(pool)
            .await
            .unwrap();
        user_id
    }

    #[tokio::test]
    async fn test_ai_interaction_logging() {
        let pool = setup_test_db().await;
        let repo = AiInteractionRepository::new(&pool);
        let user_id = create_test_user(&pool).await;

        let interaction = AiInteraction {
            id: Uuid::new_v4(),
            user_id,
            interaction_type: "TagSuggestion".to_string(),
            prompt: "Suggest tags for this contact".to_string(),
            response: "Professional, Tech Industry, Partner".to_string(),
            confidence: 0.85,
            model: "llama-3.1-8b-instruct".to_string(),
            entity_type: Some("Contact".to_string()),
            entity_id: Some(Uuid::new_v4()),
            feedback_helpful: None,
            feedback_applied: false,
            metadata: serde_json::json!({"cache_hit": false, "retries": 0}),
            created_at: Utc::now(),
            feedback_at: None,
        };

        repo.create(&interaction).await.unwrap();

        let retrieved = repo.get_by_id(interaction.id).await.unwrap();
        assert_eq!(retrieved.prompt, "Suggest tags for this contact");
        assert_eq!(retrieved.confidence, 0.85);
        assert_eq!(retrieved.model, "llama-3.1-8b-instruct");
    }

    #[tokio::test]
    async fn test_ai_interaction_feedback() {
        let pool = setup_test_db().await;
        let repo = AiInteractionRepository::new(&pool);
        let user_id = create_test_user(&pool).await;

        let interaction = AiInteraction {
            id: Uuid::new_v4(),
            user_id,
            interaction_type: "NextAction".to_string(),
            prompt: "What should I do next?".to_string(),
            response: "Schedule a follow-up meeting within 2 weeks".to_string(),
            confidence: 0.9,
            model: "llama-3.1-8b-instruct".to_string(),
            entity_type: Some("Contact".to_string()),
            entity_id: Some(Uuid::new_v4()),
            feedback_helpful: None,
            feedback_applied: false,
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
            feedback_at: None,
        };

        repo.create(&interaction).await.unwrap();

        // User provides positive feedback
        repo.update_feedback(interaction.id, true, false)
            .await
            .unwrap();

        let updated = repo.get_by_id(interaction.id).await.unwrap();
        assert_eq!(updated.feedback_helpful, Some(true));
        assert_eq!(updated.feedback_applied, false);
        assert!(updated.feedback_at.is_some());

        // User applies the suggestion
        repo.update_feedback(interaction.id, true, true)
            .await
            .unwrap();

        let applied = repo.get_by_id(interaction.id).await.unwrap();
        assert_eq!(applied.feedback_helpful, Some(true));
        assert_eq!(applied.feedback_applied, true);
    }

    #[tokio::test]
    async fn test_ai_interaction_list_by_user() {
        let pool = setup_test_db().await;
        let repo = AiInteractionRepository::new(&pool);
        let user_id = create_test_user(&pool).await;

        // Create multiple interactions
        for i in 1..=5 {
            let interaction = AiInteraction {
                id: Uuid::new_v4(),
                user_id,
                interaction_type: format!("Type{}", i),
                prompt: format!("Prompt {}", i),
                response: format!("Response {}", i),
                confidence: 0.7 + (i as f32 * 0.05),
                model: "llama-3.1-8b-instruct".to_string(),
                entity_type: None,
                entity_id: None,
                feedback_helpful: None,
                feedback_applied: false,
                metadata: serde_json::json!({}),
                created_at: Utc::now(),
                feedback_at: None,
            };
            repo.create(&interaction).await.unwrap();
        }

        let interactions = repo.list_by_user(user_id, 10, 0).await.unwrap();
        assert_eq!(interactions.len(), 5);
    }

    #[tokio::test]
    async fn test_ai_interaction_list_by_entity() {
        let pool = setup_test_db().await;
        let repo = AiInteractionRepository::new(&pool);
        let user_id = create_test_user(&pool).await;
        let entity_id = Uuid::new_v4();

        // Create interactions for specific entity
        for i in 1..=3 {
            let interaction = AiInteraction {
                id: Uuid::new_v4(),
                user_id,
                interaction_type: "ContactAnalysis".to_string(),
                prompt: format!("Analyze contact {}", i),
                response: format!("Analysis result {}", i),
                confidence: 0.8,
                model: "llama-3.1-8b-instruct".to_string(),
                entity_type: Some("Contact".to_string()),
                entity_id: Some(entity_id),
                feedback_helpful: None,
                feedback_applied: false,
                metadata: serde_json::json!({}),
                created_at: Utc::now(),
                feedback_at: None,
            };
            repo.create(&interaction).await.unwrap();
        }

        let entity_interactions = repo
            .list_by_entity("Contact", entity_id)
            .await
            .unwrap();

        assert_eq!(entity_interactions.len(), 3);
        assert!(entity_interactions
            .iter()
            .all(|i| i.entity_id == Some(entity_id)));
    }

    #[tokio::test]
    async fn test_ai_interaction_cache_tracking() {
        let pool = setup_test_db().await;
        let repo = AiInteractionRepository::new(&pool);
        let user_id = create_test_user(&pool).await;

        let cached_interaction = AiInteraction {
            id: Uuid::new_v4(),
            user_id,
            interaction_type: "TagSuggestion".to_string(),
            prompt: "repeated prompt".to_string(),
            response: "cached response".to_string(),
            confidence: 0.85,
            model: "llama-3.1-8b-instruct".to_string(),
            entity_type: None,
            entity_id: None,
            feedback_helpful: None,
            feedback_applied: false,
            metadata: serde_json::json!({"cache_hit": true, "cached_at": "2025-01-15T10:00:00Z"}),
            created_at: Utc::now(),
            feedback_at: None,
        };

        repo.create(&cached_interaction).await.unwrap();

        let retrieved = repo.get_by_id(cached_interaction.id).await.unwrap();
        assert_eq!(
            retrieved.metadata.get("cache_hit").and_then(|v| v.as_bool()),
            Some(true)
        );
    }

    #[tokio::test]
    async fn test_ai_interaction_retry_tracking() {
        let pool = setup_test_db().await;
        let repo = AiInteractionRepository::new(&pool);
        let user_id = create_test_user(&pool).await;

        let retry_interaction = AiInteraction {
            id: Uuid::new_v4(),
            user_id,
            interaction_type: "ChannelRecommendation".to_string(),
            prompt: "Which channel should I use?".to_string(),
            response: "Email is recommended".to_string(),
            confidence: 0.75,
            model: "llama-3.1-8b-instruct".to_string(),
            entity_type: None,
            entity_id: None,
            feedback_helpful: None,
            feedback_applied: false,
            metadata: serde_json::json!({
                "retries": 2,
                "first_attempt_failed": true,
                "backoff_ms": [100, 200]
            }),
            created_at: Utc::now(),
            feedback_at: None,
        };

        repo.create(&retry_interaction).await.unwrap();

        let retrieved = repo.get_by_id(retry_interaction.id).await.unwrap();
        assert_eq!(
            retrieved.metadata.get("retries").and_then(|v| v.as_i64()),
            Some(2)
        );
        assert_eq!(
            retrieved
                .metadata
                .get("first_attempt_failed")
                .and_then(|v| v.as_bool()),
            Some(true)
        );
    }

    #[tokio::test]
    async fn test_ai_interaction_delete() {
        let pool = setup_test_db().await;
        let repo = AiInteractionRepository::new(&pool);
        let user_id = create_test_user(&pool).await;

        let interaction = AiInteraction {
            id: Uuid::new_v4(),
            user_id,
            interaction_type: "Test".to_string(),
            prompt: "test prompt".to_string(),
            response: "test response".to_string(),
            confidence: 0.5,
            model: "test-model".to_string(),
            entity_type: None,
            entity_id: None,
            feedback_helpful: None,
            feedback_applied: false,
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
            feedback_at: None,
        };

        repo.create(&interaction).await.unwrap();

        // Verify it exists
        assert!(repo.get_by_id(interaction.id).await.is_ok());

        // Delete it
        repo.delete(interaction.id).await.unwrap();

        // Verify it's gone
        assert!(repo.get_by_id(interaction.id).await.is_err());
    }

    #[tokio::test]
    async fn test_ai_interaction_recent_list() {
        let pool = setup_test_db().await;
        let repo = AiInteractionRepository::new(&pool);
        let user_id = create_test_user(&pool).await;

        // Create 10 interactions
        for i in 1..=10 {
            let interaction = AiInteraction {
                id: Uuid::new_v4(),
                user_id,
                interaction_type: format!("Type{}", i),
                prompt: format!("Prompt {}", i),
                response: format!("Response {}", i),
                confidence: 0.8,
                model: "llama-3.1-8b-instruct".to_string(),
                entity_type: None,
                entity_id: None,
                feedback_helpful: None,
                feedback_applied: false,
                metadata: serde_json::json!({}),
                created_at: Utc::now(),
                feedback_at: None,
            };
            repo.create(&interaction).await.unwrap();
        }

        // Get recent 5
        let recent = repo.list_recent(5).await.unwrap();
        assert_eq!(recent.len(), 5);
    }
}
