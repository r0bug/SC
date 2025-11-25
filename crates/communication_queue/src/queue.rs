use crate::adapters::{
    EmailAdapter, EmailAdapterTrait, MockEmailAdapter, MockSmsAdapter, SmsAdapter,
    SmsAdapterTrait, SmtpEmailAdapter, SocialAdapter, TwilioSmsAdapter,
};
use crate::config::CommunicationConfig;
use anyhow::Result;
use core_domain::{CommunicationAttempt, CommunicationMethod, CommunicationStatus};
use local_store::CommunicationRepository;
use std::sync::Arc;
use tracing::{error, info, warn};

pub struct CommunicationQueue {
    email_adapter_trait: Arc<dyn EmailAdapterTrait>,
    sms_adapter_trait: Arc<dyn SmsAdapterTrait>,
    // Legacy adapters for compatibility
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
    /// Create queue with default mock adapters (for backward compatibility)
    pub fn new() -> Self {
        Self {
            email_adapter_trait: Arc::new(MockEmailAdapter::new()),
            sms_adapter_trait: Arc::new(MockSmsAdapter::new()),
            email_adapter: EmailAdapter::new(),
            sms_adapter: SmsAdapter::new(),
            social_adapter: SocialAdapter::new(),
        }
    }

    /// Create queue with configuration-based adapter selection
    pub fn with_config(config: CommunicationConfig) -> Result<Self> {
        config.log_status();

        // Select email adapter based on config
        let email_adapter_trait: Arc<dyn EmailAdapterTrait> = if config.is_real_email_available() {
            info!("Using real SMTP email adapter");
            Arc::new(SmtpEmailAdapter::from_env()?)
        } else {
            info!("Using mock email adapter");
            Arc::new(MockEmailAdapter::new())
        };

        // Select SMS adapter based on config
        let sms_adapter_trait: Arc<dyn SmsAdapterTrait> = if config.is_real_sms_available() {
            info!("Using real Twilio SMS adapter");
            Arc::new(TwilioSmsAdapter::from_env()?)
        } else {
            info!("Using mock SMS adapter");
            Arc::new(MockSmsAdapter::new())
        };

        Ok(Self {
            email_adapter_trait,
            sms_adapter_trait,
            email_adapter: EmailAdapter::new(),
            sms_adapter: SmsAdapter::new(),
            social_adapter: SocialAdapter::new(),
        })
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
            CommunicationMethod::Email => {
                // For email, we need to extract the recipient from the contact
                // In a real scenario, this would be looked up from the contact repository
                // For now, we'll use a placeholder approach
                let to = "placeholder@example.com"; // TODO: Look up from contact
                let subject = attempt.subject.as_deref().unwrap_or("No Subject");
                let body = &attempt.message;

                self.email_adapter_trait.send_email(to, subject, body).await
            }
            CommunicationMethod::SMS => {
                // For SMS, we need to extract the phone number from the contact
                // In a real scenario, this would be looked up from the contact repository
                let to = "+15555550100"; // TODO: Look up from contact

                self.sms_adapter_trait.send_sms(to, &attempt.message).await
            }
            CommunicationMethod::Social { platform } => {
                // Still use legacy adapter for social (still mocked)
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
