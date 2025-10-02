use core_domain::{Contact, AiSuggestion};
use crate::SegmindClient;
use anyhow::Result;
use uuid::Uuid;
use chrono::Utc;
use tracing::info;

pub struct SuggestionEngine {
    client: SegmindClient,
}

impl SuggestionEngine {
    pub fn new(client: SegmindClient) -> Self {
        Self { client }
    }

    pub async fn generate_contact_suggestions(&self, contact: &Contact) -> Result<Vec<AiSuggestion>> {
        info!("Generating AI suggestions for contact {}", contact.id);

        let prompt = format!(
            "Analyze contact: {} {} from {}",
            contact.first_name,
            contact.last_name.as_deref().unwrap_or(""),
            contact.organization.as_deref().unwrap_or("Unknown")
        );

        let response = self.client.generate_suggestion(&prompt).await?;

        let suggestion = AiSuggestion {
            id: Uuid::new_v4(),
            contact_id: Some(contact.id),
            suggestion_type: "contact_enrichment".to_string(),
            content: response.text,
            confidence: response.confidence,
            applied: false,
            created_at: Utc::now(),
        };

        Ok(vec![suggestion])
    }

    pub async fn suggest_next_action(&self, contact: &Contact) -> Result<AiSuggestion> {
        info!("Suggesting next action for contact {}", contact.id);

        let analysis = self.client.analyze_contact_data(&serde_json::to_string(contact)?).await?;

        Ok(AiSuggestion {
            id: Uuid::new_v4(),
            contact_id: Some(contact.id),
            suggestion_type: "next_action".to_string(),
            content: analysis.next_action,
            confidence: analysis.relationship_strength,
            applied: false,
            created_at: Utc::now(),
        })
    }

    pub async fn suggest_tags(&self, contact: &Contact) -> Result<Vec<String>> {
        info!("Suggesting tags for contact {}", contact.id);

        let analysis = self.client.analyze_contact_data(&serde_json::to_string(contact)?).await?;

        Ok(analysis.suggested_tags)
    }
}