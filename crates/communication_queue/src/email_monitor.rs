use anyhow::Result;
use async_imap::types::Fetch;
use async_native_tls::TlsConnector;
use core_domain::system_user_id;
use futures::StreamExt;
use local_store::ContactRepository;
use mail_parser::MessageParser;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncReadCompatExt;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Configuration for IMAP email monitoring
#[derive(Clone, Debug)]
pub struct ImapConfig {
    pub server: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub poll_interval_secs: u64,
}

impl ImapConfig {
    /// Load from environment variables
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            server: std::env::var("IMAP_SERVER")
                .map_err(|_| anyhow::anyhow!("IMAP_SERVER not set"))?,
            port: std::env::var("IMAP_PORT")
                .unwrap_or_else(|_| "993".to_string())
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid IMAP_PORT"))?,
            username: std::env::var("IMAP_USERNAME")
                .map_err(|_| anyhow::anyhow!("IMAP_USERNAME not set"))?,
            password: std::env::var("IMAP_PASSWORD")
                .map_err(|_| anyhow::anyhow!("IMAP_PASSWORD not set"))?,
            poll_interval_secs: std::env::var("IMAP_POLL_INTERVAL")
                .unwrap_or_else(|_| "300".to_string()) // Default 5 minutes
                .parse()
                .unwrap_or(300),
        })
    }
}

/// Email monitor that polls IMAP for new messages
pub struct EmailMonitor {
    config: ImapConfig,
    db_pool: Arc<sqlx::SqlitePool>,
    ai_analyzer: Arc<crate::email_ai_analyzer::EmailAIAnalyzer>,
}

impl EmailMonitor {
    pub fn new(config: ImapConfig, db_pool: Arc<sqlx::SqlitePool>) -> Self {
        let ai_analyzer = Arc::new(crate::email_ai_analyzer::EmailAIAnalyzer::new(db_pool.clone()));
        Self { config, db_pool, ai_analyzer }
    }

    /// Start monitoring emails in a background task
    pub async fn start(self: Arc<Self>) {
        info!("📧 Starting email monitor (polling every {}s)", self.config.poll_interval_secs);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                tokio::time::Duration::from_secs(self.config.poll_interval_secs)
            );

            loop {
                interval.tick().await;

                if let Err(e) = self.check_new_emails().await {
                    error!("❌ Email monitor error: {}", e);
                    // Continue monitoring despite errors
                }
            }
        });
    }

    /// Check for new emails via IMAP
    async fn check_new_emails(&self) -> Result<()> {
        info!("📬 Checking for new emails...");

        // Connect to IMAP server
        let tcp_stream = TcpStream::connect((self.config.server.as_str(), self.config.port)).await?;
        let tls = TlsConnector::new();
        let tls_stream = tls.connect(&self.config.server, tcp_stream.compat()).await?;

        let client = async_imap::Client::new(tls_stream);

        // Login
        let mut imap_session = client
            .login(&self.config.username, &self.config.password)
            .await
            .map_err(|e| anyhow::anyhow!("IMAP login failed: {:?}", e))?;

        // Select INBOX
        imap_session.select("INBOX").await?;

        // Search for unread emails
        let unseen_uids = imap_session.uid_search("UNSEEN").await?;

        if unseen_uids.is_empty() {
            info!("✅ No new emails");
            imap_session.logout().await?;
            return Ok(());
        }

        info!("📨 Found {} new email(s)", unseen_uids.len());

        // Fetch email details
        let uid_set = unseen_uids.iter()
            .map(|uid| uid.to_string())
            .collect::<Vec<_>>()
            .join(",");

        let messages = imap_session
            .uid_fetch(uid_set, "RFC822")
            .await?;

        // Collect stream into vector
        let messages: Vec<_> = messages.collect().await;

        // Process each email
        for msg_result in messages.iter() {
            match msg_result {
                Ok(msg) => {
                    if let Err(e) = self.process_email(msg).await {
                        error!("Failed to process email: {}", e);
                    }
                }
                Err(e) => {
                    error!("Failed to fetch email: {}", e);
                }
            }
        }

        imap_session.logout().await?;
        Ok(())
    }

    /// Process a single email message
    async fn process_email(&self, fetch: &Fetch) -> Result<()> {
        // Parse email
        let body = fetch.body()
            .ok_or_else(|| anyhow::anyhow!("No email body"))?;

        let parser = MessageParser::default();
        let message = parser.parse(body)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse email"))?;

        // Extract email details
        let from_address = message.from()
            .and_then(|addrs| addrs.first())
            .and_then(|addr| addr.address())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unknown@example.com".to_string());

        let from_name = message.from()
            .and_then(|addrs| addrs.first())
            .and_then(|addr| addr.name())
            .map(|s| s.to_string());

        let to_address = message.to()
            .and_then(|addrs| addrs.first())
            .and_then(|addr| addr.address())
            .map(|s| s.to_string())
            .unwrap_or_else(|| self.config.username.clone());

        let subject = message.subject()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "(No Subject)".to_string());

        let body_text = message.body_text(0)
            .map(|s| s.to_string());

        let body_html = message.body_html(0)
            .map(|s| s.to_string());

        let message_id = message.message_id()
            .map(|s| s.to_string());

        info!("📧 Processing email from: {}", from_address);
        info!("   Subject: {}", subject);

        // Find or create contact
        let contact_repo = ContactRepository::new(&self.db_pool);
        let placeholder_user = system_user_id(); // System user for email-triggered contacts

        let contact_id = match contact_repo.search(&from_address, placeholder_user).await {
            Ok(contacts) if !contacts.is_empty() => {
                Some(contacts[0].id)
            }
            _ => {
                // Auto-create contact from email
                let new_contact = core_domain::Contact {
                    id: Uuid::new_v4(),
                    first_name: from_name.clone().unwrap_or_else(|| {
                        from_address.split('@').next().unwrap_or("Unknown").to_string()
                    }),
                    last_name: None,
                    email: Some(from_address.clone()),
                    phone: None,
                    organization: None,
                    title: None,
                    notes: Some(format!("Auto-created from email: {}", subject)),
                    tags: Vec::new(),
                    social_handles: Vec::new(),
                    projects: Vec::new(),
                    groups: Vec::new(),
                    metadata: serde_json::json!({}),
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    created_by: system_user_id(), // System user for auto-created contacts
                    version: 1,
                    last_synced_at: None,
                };

                match contact_repo.create(&new_contact).await {
                    Ok(_) => {
                        info!("✨ Auto-created contact for {}", from_address);
                        Some(new_contact.id)
                    }
                    Err(e) => {
                        warn!("Failed to create contact: {}", e);
                        None
                    }
                }
            }
        };

        // Store email in database
        let email_id = Uuid::new_v4();
        let now = chrono::Utc::now();

        // Create let bindings for temporary values to extend their lifetime
        let email_id_str = email_id.to_string();
        let contact_id_str = contact_id.map(|id| id.to_string());
        let imported_at_str = now.to_rfc3339();
        let message_timestamp = now.timestamp_millis();

        sqlx::query!(
            r#"
            INSERT INTO email_history (
                id, contact_id, from_address, from_name, to_address,
                subject, body_text, body_html, message_date, message_type,
                message_id, read_status, imported_at, imported_by, source_server
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            email_id_str,
            contact_id_str,
            from_address,
            from_name,
            to_address,
            subject,
            body_text,
            body_html,
            message_timestamp,
            1, // 1 = received
            message_id,
            0, // 0 = unread
            imported_at_str,
            "imap_sync",
            self.config.server
        )
        .execute(self.db_pool.as_ref())
        .await?;

        info!("✅ Email stored in database: {}", email_id);

        // Analyze email importance with AI
        let body_for_analysis = body_text.as_deref().unwrap_or("");
        if let Err(e) = self.ai_analyzer.analyze_email(
            email_id,
            &from_address,
            &subject,
            body_for_analysis,
        ).await {
            warn!("Failed to analyze email importance: {}", e);
        }

        Ok(())
    }
}
