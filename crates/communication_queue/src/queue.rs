use crate::adapters::{EmailAdapter, SmsAdapter, SocialAdapter};
use anyhow::Result;
use core_domain::{CommunicationAttempt, CommunicationMethod, CommunicationStatus};
use local_store::CommunicationRepository;
use tracing::{error, info, warn};

pub struct CommunicationQueue {
    email_adapter: EmailAdapter,
    sms_adapter: SmsAdapter,
    social_adapter: SocialAdapter,
}

impl Default for CommunicationQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl CommunicationQueue {
    pub fn new() -> Self {
        Self {
            email_adapter: EmailAdapter::new(),
            sms_adapter: SmsAdapter::new(),
            social_adapter: SocialAdapter::new(),
        }
    }

    pub async fn process_attempt(
        &self,
        attempt: &mut CommunicationAttempt,
        repo: &CommunicationRepository<'_>,
    ) -> Result<()> {
        info!(
            "Processing communication attempt {} for contact {}",
            attempt.id, attempt.contact_id
        );

        let result = match &attempt.method {
            CommunicationMethod::Email => self.email_adapter.send(attempt).await,
            CommunicationMethod::SMS => self.sms_adapter.send(attempt).await,
            CommunicationMethod::Social { platform } => {
                self.social_adapter.send(attempt, platform).await
            }
        };

        match result {
            Ok(_) => {
                info!("Successfully sent communication {}", attempt.id);
                let now = chrono::Utc::now();
                repo.update_status(attempt.id, CommunicationStatus::Sent, Some(now))
                    .await
                    .map_err(|e| anyhow::anyhow!(e))?;
            }
            Err(e) => {
                error!("Failed to send communication {}: {}", attempt.id, e);
                attempt.retry_count += 1;

                let status = if attempt.retry_count < 3 {
                    warn!("Marking for retry (attempt {})", attempt.retry_count);
                    CommunicationStatus::Retrying
                } else {
                    error!("Max retries reached, marking as failed");
                    CommunicationStatus::Failed {
                        reason: e.to_string(),
                    }
                };

                let now = chrono::Utc::now();
                repo.update_retry_count(attempt.id, attempt.retry_count)
                    .await
                    .map_err(|e| anyhow::anyhow!(e))?;
                repo.update_status(attempt.id, status, Some(now))
                    .await
                    .map_err(|e| anyhow::anyhow!(e))?;
            }
        }

        Ok(())
    }

    pub async fn process_pending(&self, repo: &CommunicationRepository<'_>) -> Result<()> {
        let pending = repo.list_pending().await.map_err(|e| anyhow::anyhow!(e))?;

        info!("Found {} pending communication attempts", pending.len());

        for mut attempt in pending {
            if let Some(scheduled) = attempt.scheduled_at {
                if scheduled > chrono::Utc::now() {
                    continue;
                }
            }

            self.process_attempt(&mut attempt, repo).await?;
        }

        Ok(())
    }
}
