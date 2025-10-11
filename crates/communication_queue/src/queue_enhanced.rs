use core_domain::{CommunicationAttempt, CommunicationStatus, CommunicationMethod};
use local_store::CommunicationRepository;
use crate::adapters::{
    EmailAdapterTrait, SmsAdapterTrait, SocialAdapterTrait,
    MockEmailAdapter, MockSmsAdapter, MockSocialAdapter
};
use anyhow::Result;
use tracing::{info, warn, error};
use chrono::{Utc, Duration};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub channel: String,
    pub max_per_minute: usize,
    pub max_per_hour: usize,
    pub max_per_day: usize,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            channel: "default".to_string(),
            max_per_minute: 10,
            max_per_hour: 100,
            max_per_day: 1000,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BackoffConfig {
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub multiplier: f32,
    pub max_retries: u32,
}

impl Default for BackoffConfig {
    fn default() -> Self {
        Self {
            initial_delay_ms: 1000,  // 1 second
            max_delay_ms: 60000,     // 1 minute
            multiplier: 2.0,
            max_retries: 5,
        }
    }
}

struct RateLimiter {
    counts: Arc<Mutex<HashMap<String, RateLimitState>>>,
}

#[derive(Debug, Clone)]
struct RateLimitState {
    minute_count: usize,
    minute_reset: chrono::DateTime<chrono::Utc>,
    hour_count: usize,
    hour_reset: chrono::DateTime<chrono::Utc>,
    day_count: usize,
    day_reset: chrono::DateTime<chrono::Utc>,
}

impl RateLimiter {
    fn new() -> Self {
        Self {
            counts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn can_send(&self, channel: &str, config: &RateLimitConfig) -> bool {
        let mut counts = self.counts.lock().await;
        let now = Utc::now();

        let state = counts.entry(channel.to_string()).or_insert_with(|| {
            RateLimitState {
                minute_count: 0,
                minute_reset: now + Duration::minutes(1),
                hour_count: 0,
                hour_reset: now + Duration::hours(1),
                day_count: 0,
                day_reset: now + Duration::days(1),
            }
        });

        // Reset counters if time windows have passed
        if now >= state.minute_reset {
            state.minute_count = 0;
            state.minute_reset = now + Duration::minutes(1);
        }
        if now >= state.hour_reset {
            state.hour_count = 0;
            state.hour_reset = now + Duration::hours(1);
        }
        if now >= state.day_reset {
            state.day_count = 0;
            state.day_reset = now + Duration::days(1);
        }

        // Check limits
        if state.minute_count >= config.max_per_minute {
            warn!("Rate limit exceeded for channel {} (minute)", channel);
            return false;
        }
        if state.hour_count >= config.max_per_hour {
            warn!("Rate limit exceeded for channel {} (hour)", channel);
            return false;
        }
        if state.day_count >= config.max_per_day {
            warn!("Rate limit exceeded for channel {} (day)", channel);
            return false;
        }

        // Increment counters
        state.minute_count += 1;
        state.hour_count += 1;
        state.day_count += 1;

        true
    }
}

pub struct EnhancedCommunicationQueue {
    email_adapter: Box<dyn EmailAdapterTrait>,
    sms_adapter: Box<dyn SmsAdapterTrait>,
    social_adapter: Box<dyn SocialAdapterTrait>,
    rate_limiter: RateLimiter,
    rate_configs: HashMap<String, RateLimitConfig>,
    backoff_config: BackoffConfig,
    batch_size: usize,
}

impl Default for EnhancedCommunicationQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl EnhancedCommunicationQueue {
    pub fn new() -> Self {
        let mut rate_configs = HashMap::new();
        rate_configs.insert("email".to_string(), RateLimitConfig {
            channel: "email".to_string(),
            max_per_minute: 20,
            max_per_hour: 200,
            max_per_day: 2000,
        });
        rate_configs.insert("sms".to_string(), RateLimitConfig {
            channel: "sms".to_string(),
            max_per_minute: 10,
            max_per_hour: 100,
            max_per_day: 500,
        });
        rate_configs.insert("social".to_string(), RateLimitConfig {
            channel: "social".to_string(),
            max_per_minute: 30,
            max_per_hour: 300,
            max_per_day: 3000,
        });

        Self {
            email_adapter: Box::new(MockEmailAdapter::new()),
            sms_adapter: Box::new(MockSmsAdapter::new()),
            social_adapter: Box::new(MockSocialAdapter::new()),
            rate_limiter: RateLimiter::new(),
            rate_configs,
            backoff_config: BackoffConfig::default(),
            batch_size: 10,
        }
    }

    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }

    pub fn with_backoff_config(mut self, config: BackoffConfig) -> Self {
        self.backoff_config = config;
        self
    }

    pub async fn process_batch(&self, repo: &CommunicationRepository<'_>) -> Result<usize> {
        let pending = repo.list_pending().await?;

        if pending.is_empty() {
            return Ok(0);
        }

        // Limit to batch size
        let to_process: Vec<_> = pending.into_iter().take(self.batch_size).collect();

        info!("Processing batch of {} communications", to_process.len());
        let mut processed = 0;

        for mut attempt in to_process {
            let channel = match &attempt.method {
                CommunicationMethod::Email => "email",
                CommunicationMethod::SMS => "sms",
                CommunicationMethod::Social { .. } => "social",
            };

            // Check rate limits
            let default_config = RateLimitConfig::default();
            let rate_config = self.rate_configs.get(channel)
                .unwrap_or(&default_config);

            if !self.rate_limiter.can_send(channel, rate_config).await {
                warn!("Skipping {} due to rate limit", attempt.id);
                continue;
            }

            // Apply exponential backoff if this is a retry
            if attempt.retry_count > 0 {
                let delay = self.calculate_backoff_delay(attempt.retry_count as u32);
                tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
            }

            // Process the communication
            match self.process_single(&mut attempt, repo).await {
                Ok(_) => {
                    processed += 1;
                    info!("Successfully processed communication {}", attempt.id);
                }
                Err(e) => {
                    error!("Failed to process communication {}: {}", attempt.id, e);

                    // Update retry count and status
                    attempt.retry_count += 1;

                    if attempt.retry_count >= self.backoff_config.max_retries as i32 {
                        attempt.status = CommunicationStatus::Failed { reason: "Max retries exceeded".to_string() };
                        warn!("Communication {} exceeded max retries", attempt.id);
                    }

                    repo.update_status(attempt.id, attempt.status.clone(), attempt.attempted_at).await?;
                    repo.update_retry_count(attempt.id, attempt.retry_count).await?;
                }
            }
        }

        Ok(processed)
    }

    async fn process_single(
        &self,
        attempt: &mut CommunicationAttempt,
        repo: &CommunicationRepository<'_>,
    ) -> Result<()> {
        attempt.attempted_at = Some(Utc::now());
        repo.update_status(attempt.id, CommunicationStatus::Retrying, attempt.attempted_at).await?;

        let result = match &attempt.method {
            CommunicationMethod::Email => {
                self.email_adapter.send_email(
                    "", // Recipient would come from contact
                    attempt.subject.as_deref().unwrap_or("No Subject"),
                    &attempt.message,
                ).await
            }
            CommunicationMethod::SMS => {
                self.sms_adapter.send_sms(
                    "", // Phone would come from contact
                    &attempt.message,
                ).await
            }
            CommunicationMethod::Social { platform } => {
                self.social_adapter.send_message(
                    platform,
                    "", // Handle would come from contact
                    &attempt.message,
                ).await
            }
        };

        match result {
            Ok(_) => {
                attempt.status = CommunicationStatus::Sent;
                repo.update_status(attempt.id, CommunicationStatus::Sent, attempt.attempted_at).await?;
                Ok(())
            }
            Err(e) => {
                attempt.status = CommunicationStatus::Failed { reason: e.to_string() };
                repo.update_status(attempt.id, attempt.status.clone(), attempt.attempted_at).await?;
                Err(e)
            }
        }
    }

    fn calculate_backoff_delay(&self, retry_count: u32) -> u64 {
        let delay = self.backoff_config.initial_delay_ms as f32
            * self.backoff_config.multiplier.powi(retry_count as i32 - 1);

        delay.min(self.backoff_config.max_delay_ms as f32) as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backoff_calculation() {
        let queue = EnhancedCommunicationQueue::new();

        assert_eq!(queue.calculate_backoff_delay(1), 1000);  // 1 second
        assert_eq!(queue.calculate_backoff_delay(2), 2000);  // 2 seconds
        assert_eq!(queue.calculate_backoff_delay(3), 4000);  // 4 seconds
        assert_eq!(queue.calculate_backoff_delay(4), 8000);  // 8 seconds
        assert_eq!(queue.calculate_backoff_delay(5), 16000); // 16 seconds
        assert_eq!(queue.calculate_backoff_delay(10), 60000); // Capped at max
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        let limiter = RateLimiter::new();
        let config = RateLimitConfig {
            channel: "test".to_string(),
            max_per_minute: 2,
            max_per_hour: 10,
            max_per_day: 100,
        };

        // Should allow first two
        assert!(limiter.can_send("test", &config).await);
        assert!(limiter.can_send("test", &config).await);

        // Should block third (minute limit)
        assert!(!limiter.can_send("test", &config).await);
    }
}