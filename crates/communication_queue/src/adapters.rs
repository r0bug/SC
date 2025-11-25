use anyhow::Result;
use async_trait::async_trait;
use tracing::{info, warn};
use base64::{Engine as _, engine::general_purpose};

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

/// Twilio SMS adapter - real SMS sending via Twilio REST API
#[derive(Debug)]
pub struct TwilioSmsAdapter {
    account_sid: String,
    auth_token: String,
    from_number: String,
    client: reqwest::Client,
}

impl TwilioSmsAdapter {
    pub fn new(account_sid: String, auth_token: String, from_number: String) -> Self {
        Self {
            account_sid,
            auth_token,
            from_number,
            client: reqwest::Client::new(),
        }
    }

    /// Create from environment variables
    pub fn from_env() -> Result<Self> {
        let account_sid = std::env::var("TWILIO_ACCOUNT_SID")
            .map_err(|_| anyhow::anyhow!("TWILIO_ACCOUNT_SID not set"))?;
        let auth_token = std::env::var("TWILIO_AUTH_TOKEN")
            .map_err(|_| anyhow::anyhow!("TWILIO_AUTH_TOKEN not set"))?;
        let from_number = std::env::var("TWILIO_PHONE_NUMBER")
            .map_err(|_| anyhow::anyhow!("TWILIO_PHONE_NUMBER not set"))?;

        Ok(Self::new(account_sid, auth_token, from_number))
    }

    fn build_auth_header(&self) -> String {
        let credentials = format!("{}:{}", self.account_sid, self.auth_token);
        let encoded = general_purpose::STANDARD.encode(credentials.as_bytes());
        format!("Basic {}", encoded)
    }
}

#[async_trait]
impl SmsAdapterTrait for TwilioSmsAdapter {
    async fn send_sms(&self, to: &str, message: &str) -> Result<()> {
        info!("[TWILIO] Sending SMS to {} from {}", to, self.from_number);

        // Twilio API endpoint
        let url = format!(
            "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
            self.account_sid
        );

        // Prepare form data
        let params = [
            ("To", to),
            ("From", &self.from_number),
            ("Body", message),
        ];

        // Make request
        let response = self.client
            .post(&url)
            .header("Authorization", self.build_auth_header())
            .form(&params)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        if status.is_success() {
            info!("[TWILIO] SMS sent successfully to {}", to);
            info!("[TWILIO] Response: {}", body);
            Ok(())
        } else {
            warn!("[TWILIO] Failed to send SMS. Status: {}", status);
            warn!("[TWILIO] Error response: {}", body);
            anyhow::bail!("Twilio API error: {} - {}", status, body)
        }
    }
}

/// SMTP Email adapter - real email sending via SMTP
#[derive(Debug)]
pub struct SmtpEmailAdapter {
    smtp_server: String,
    smtp_port: u16,
    smtp_username: String,
    smtp_password: String,
    from_address: String,
    from_name: String,
}

impl SmtpEmailAdapter {
    pub fn new(
        smtp_server: String,
        smtp_port: u16,
        smtp_username: String,
        smtp_password: String,
        from_address: String,
        from_name: String,
    ) -> Self {
        Self {
            smtp_server,
            smtp_port,
            smtp_username,
            smtp_password,
            from_address,
            from_name,
        }
    }

    /// Create from environment variables
    pub fn from_env() -> Result<Self> {
        let smtp_server = std::env::var("SMTP_SERVER")
            .map_err(|_| anyhow::anyhow!("SMTP_SERVER not set"))?;
        let smtp_port = std::env::var("SMTP_PORT")
            .unwrap_or_else(|_| "587".to_string())
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid SMTP_PORT"))?;
        let smtp_username = std::env::var("SMTP_USERNAME")
            .map_err(|_| anyhow::anyhow!("SMTP_USERNAME not set"))?;
        let smtp_password = std::env::var("SMTP_PASSWORD")
            .map_err(|_| anyhow::anyhow!("SMTP_PASSWORD not set"))?;
        let from_address = std::env::var("SMTP_FROM_ADDRESS")
            .map_err(|_| anyhow::anyhow!("SMTP_FROM_ADDRESS not set"))?;
        let from_name = std::env::var("SMTP_FROM_NAME")
            .unwrap_or_else(|_| "SagensContact".to_string());

        Ok(Self::new(
            smtp_server,
            smtp_port,
            smtp_username,
            smtp_password,
            from_address,
            from_name,
        ))
    }
}

#[async_trait]
impl EmailAdapterTrait for SmtpEmailAdapter {
    async fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<()> {
        use lettre::{
            message::header::ContentType, transport::smtp::authentication::Credentials, Message,
            SmtpTransport, Transport,
        };

        info!("[SMTP] Sending email to {}", to);
        info!("[SMTP] Subject: {}", subject);

        // Build email message
        let email = Message::builder()
            .from(
                format!("{} <{}>", self.from_name, self.from_address)
                    .parse()
                    .map_err(|e| anyhow::anyhow!("Invalid from address: {}", e))?,
            )
            .to(to
                .parse()
                .map_err(|e| anyhow::anyhow!("Invalid to address: {}", e))?)
            .subject(subject)
            .header(ContentType::TEXT_PLAIN)
            .body(body.to_string())
            .map_err(|e| anyhow::anyhow!("Failed to build email: {}", e))?;

        // Create SMTP client
        let creds = Credentials::new(self.smtp_username.clone(), self.smtp_password.clone());

        let mailer = SmtpTransport::starttls_relay(&self.smtp_server)
            .map_err(|e| anyhow::anyhow!("Failed to connect to SMTP server: {}", e))?
            .port(self.smtp_port)
            .credentials(creds)
            .build();

        // Send email
        match mailer.send(&email) {
            Ok(_) => {
                info!("[SMTP] Email sent successfully to {}", to);
                Ok(())
            }
            Err(e) => {
                warn!("[SMTP] Failed to send email: {}", e);
                anyhow::bail!("SMTP send error: {}", e)
            }
        }
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
