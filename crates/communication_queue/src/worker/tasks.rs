use crate::{CommunicationQueue, NagScheduler};
use anyhow::Result;
use chrono::{DateTime, Utc};
use core_domain::{AiInsight, AiInsightEntityType, AiInsightType};
use local_store::{
    AiInsightRepository, CommunicationRepository, ContactRepository, LocalStore,
    SearchHistoryRepository,
};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};
use uuid::Uuid;

/// Task for processing communication queue
pub struct CommunicationTask {
    store: Arc<LocalStore>,
    queue: Arc<CommunicationQueue>,
    interval: Duration,
    sync_service_url: Option<String>,
}

impl CommunicationTask {
    pub fn new(store: Arc<LocalStore>, sync_service_url: Option<String>) -> Self {
        Self {
            store,
            queue: Arc::new(CommunicationQueue::new()),
            interval: Duration::from_secs(30),
            sync_service_url,
        }
    }

    pub async fn run(&self) -> Result<()> {
        let mut cycle = 0;

        loop {
            cycle += 1;
            info!("Communication task cycle {}", cycle);

            let repo = CommunicationRepository::new(self.store.pool());

            match self.queue.process_pending(&repo).await {
                Ok(()) => {
                    info!("Processed communications in cycle {}", cycle);

                    // Send callback to sync service if configured
                    if let Some(url) = &self.sync_service_url {
                        self.send_sync_callback(url, 0).await?; // TODO: track actual count
                    }
                }
                Err(e) => {
                    error!("Error processing communications: {}", e);
                }
            }

            sleep(self.interval).await;
        }
    }

    async fn send_sync_callback(&self, url: &str, processed: usize) -> Result<()> {
        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/api/worker/communication-processed", url))
            .json(&serde_json::json!({
                "processed": processed,
                "timestamp": Utc::now(),
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            warn!("Sync service callback failed: {}", response.status());
        }

        Ok(())
    }
}

/// Task for scheduling nag reminders
pub struct NagReminderTask {
    scheduler: Arc<tokio::sync::Mutex<NagScheduler>>,
    interval: Duration,
}

impl NagReminderTask {
    pub async fn new() -> Result<Self> {
        let mut scheduler = NagScheduler::new().await?;
        scheduler.schedule_nag_check().await?;
        scheduler.start().await?;

        Ok(Self {
            scheduler: Arc::new(tokio::sync::Mutex::new(scheduler)),
            interval: Duration::from_secs(300), // Check every 5 minutes
        })
    }

    pub async fn run(&self) -> Result<()> {
        let mut cycle = 0;

        loop {
            cycle += 1;
            info!("Nag reminder task cycle {}", cycle);

            // The scheduler runs its own internal loop
            // We just monitor its health here
            {
                let _scheduler = self.scheduler.lock().await;
                info!("Nag scheduler is running, monitoring health");
            }

            sleep(self.interval).await;
        }
    }
}

/// Task for generating AI suggestions based on search history
pub struct SuggestionTask {
    store: Arc<LocalStore>,
    interval: Duration,
    last_run: Arc<tokio::sync::Mutex<Option<DateTime<Utc>>>>,
    processed_searches: Arc<tokio::sync::Mutex<HashSet<Uuid>>>,
}

impl SuggestionTask {
    pub fn new(store: Arc<LocalStore>) -> Self {
        Self {
            store,
            interval: Duration::from_secs(600), // Run every 10 minutes
            last_run: Arc::new(tokio::sync::Mutex::new(None)),
            processed_searches: Arc::new(tokio::sync::Mutex::new(HashSet::new())),
        }
    }

    pub async fn run(&self) -> Result<()> {
        let mut cycle = 0;

        loop {
            cycle += 1;
            info!("Suggestion generator task cycle {}", cycle);

            match self.generate_suggestions().await {
                Ok(count) => {
                    info!("Generated {} AI suggestions in cycle {}", count, cycle);

                    // Update last run time
                    let mut last_run = self.last_run.lock().await;
                    *last_run = Some(Utc::now());
                }
                Err(e) => {
                    error!("Error generating suggestions: {}", e);
                }
            }

            sleep(self.interval).await;
        }
    }

    async fn generate_suggestions(&self) -> Result<usize> {
        let search_repo = SearchHistoryRepository::new(self.store.pool());
        let insight_repo = AiInsightRepository::new(self.store.pool());
        let contact_repo = ContactRepository::new(self.store.pool());

        // Get recent search history (last 100 searches)
        // Using placeholder user ID for now
        let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap();
        let searches = search_repo.list_by_user(user_id, 100, 0).await?;

        let mut processed = self.processed_searches.lock().await;
        let mut suggestions_created = 0;

        for search in searches {
            // Skip if already processed
            if processed.contains(&search.id) {
                continue;
            }

            // Analyze search patterns (mock implementation)
            if search.query.len() > 3 && search.result_count == 0 {
                // No results found - suggest alternative searches
                let insight = AiInsight {
                    id: Uuid::new_v4(),
                    entity_type: AiInsightEntityType::Contact,
                    entity_id: search.clicked_result_id.unwrap_or(Uuid::nil()),
                    insight_type: AiInsightType::ContentSummary,
                    content: format!(
                        "Your search '{}' returned no results. Try searching for partial names or email domains.",
                        search.query
                    ),
                    confidence: 0.7,
                    prompt_template: "search_analyzer".to_string(),
                    response_cached: false,
                    feedback: None,
                    applied: false,
                    applied_at: None,
                    created_at: Utc::now(),
                    created_by: user_id,
                };

                insight_repo.create(&insight).await?;
                suggestions_created += 1;
            } else if search.result_count > 20 {
                // Too many results - suggest refinement
                let insight = AiInsight {
                    id: Uuid::new_v4(),
                    entity_type: AiInsightEntityType::Contact,
                    entity_id: search.clicked_result_id.unwrap_or(Uuid::nil()),
                    insight_type: AiInsightType::NextAction,
                    content: format!(
                        "Your search '{}' returned {} results. Consider adding more specific criteria.",
                        search.query, search.result_count
                    ),
                    confidence: 0.8,
                    prompt_template: "search_analyzer".to_string(),
                    response_cached: false,
                    feedback: None,
                    applied: false,
                    applied_at: None,
                    created_at: Utc::now(),
                    created_by: user_id,
                };

                insight_repo.create(&insight).await?;
                suggestions_created += 1;
            }

            // Mark as processed
            processed.insert(search.id);
        }

        // Analyze contact patterns for suggestions
        let contacts = contact_repo.list(10, 0).await?;

        for contact in contacts {
            // Check if contact has missing information
            if contact.email.is_none() && contact.phone.is_none() {
                let existing = insight_repo
                    .list_by_entity(AiInsightEntityType::Contact, contact.id)
                    .await?;

                // Don't create duplicate insights
                let has_similar = existing
                    .iter()
                    .any(|i| i.content.contains("missing contact information"));

                if !has_similar {
                    let insight = AiInsight {
                        id: Uuid::new_v4(),
                        entity_type: AiInsightEntityType::Contact,
                        entity_id: contact.id,
                        insight_type: AiInsightType::NextAction,
                        content: format!(
                            "Contact {} {} is missing contact information. Consider adding email or phone.",
                            contact.first_name,
                            contact.last_name.as_deref().unwrap_or("")
                        ),
                        confidence: 0.9,
                        prompt_template: "data_quality_analyzer".to_string(),
                        response_cached: false,
                        feedback: None,
                        applied: false,
                        applied_at: None,
                        created_at: Utc::now(),
                        created_by: user_id,
                    };

                    insight_repo.create(&insight).await?;
                    suggestions_created += 1;
                }
            }
        }

        Ok(suggestions_created)
    }
}
