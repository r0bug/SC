use anyhow::Result;
use async_trait::async_trait;
use tracing::{info, warn};

// Trait definitions
#[async_trait]
pub trait EmailAdapterTrait: Send + Sync {
    async fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<()>;
}

#[async_trait]
pub trait SmsAdapterTrait: Send + Sync {
    async fn send_sms(&self, to: &str, message: &str) -> Result<()>;
}

#[async_trait]
pub trait SocialAdapterTrait: Send + Sync {
    async fn send_message(&self, platform: &str, to: &str, message: &str) -> Result<()>;
}

// Mock implementations for alpha release
pub struct MockEmailAdapter;

impl Default for MockEmailAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl MockEmailAdapter {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl EmailAdapterTrait for MockEmailAdapter {
    async fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<()> {
        info!("[MOCK] Sending email to {}", to);
        info!("[MOCK] Subject: {}", subject);
        info!("[MOCK] Body: {}", body);

        if body.contains("test-fail") {
            warn!("[MOCK] Simulating email failure");
            anyhow::bail!("Mock email send failure");
        }

        info!("[MOCK] Email sent successfully (deterministic mock)");
        Ok(())
    }
}

pub struct MockSmsAdapter;

impl Default for MockSmsAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl MockSmsAdapter {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl SmsAdapterTrait for MockSmsAdapter {
    async fn send_sms(&self, to: &str, message: &str) -> Result<()> {
        info!("[MOCK] Sending SMS to {}", to);
        info!("[MOCK] Message: {}", message);

        if message.len() > 160 {
            warn!("[MOCK] Message exceeds SMS length, will be split");
        }

        info!("[MOCK] SMS sent successfully (deterministic mock)");
        Ok(())
    }
}

pub struct MockSocialAdapter;

impl Default for MockSocialAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl MockSocialAdapter {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl SocialAdapterTrait for MockSocialAdapter {
    async fn send_message(&self, platform: &str, to: &str, message: &str) -> Result<()> {
        info!("[MOCK] Sending {} message to {}", platform, to);
        info!("[MOCK] Message: {}", message);

        match platform {
            "twitter" | "linkedin" | "facebook" => {
                info!(
                    "[MOCK] {} message sent successfully (deterministic mock)",
                    platform
                );
                Ok(())
            }
            _ => {
                warn!("[MOCK] Unsupported platform: {}", platform);
                anyhow::bail!("Unsupported social platform: {}", platform)
            }
        }
    }
}

// Legacy structs for backward compatibility with queue.rs
pub struct EmailAdapter;

impl Default for EmailAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl EmailAdapter {
    pub fn new() -> Self {
        Self
    }

    pub async fn send(&self, _attempt: &core_domain::CommunicationAttempt) -> Result<()> {
        info!("[MOCK] Legacy email adapter called");
        Ok(())
    }
}

pub struct SmsAdapter;

impl Default for SmsAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl SmsAdapter {
    pub fn new() -> Self {
        Self
    }

    pub async fn send(&self, _attempt: &core_domain::CommunicationAttempt) -> Result<()> {
        info!("[MOCK] Legacy SMS adapter called");
        Ok(())
    }
}

pub struct SocialAdapter;

impl Default for SocialAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl SocialAdapter {
    pub fn new() -> Self {
        Self
    }

    pub async fn send(
        &self,
        _attempt: &core_domain::CommunicationAttempt,
        platform: &str,
    ) -> Result<()> {
        info!("[MOCK] Legacy social adapter called for {}", platform);
        Ok(())
    }
}
