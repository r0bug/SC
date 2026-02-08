use anyhow::Result;
use async_native_tls::TlsConnector;
use futures::StreamExt;
use local_store::repositories::{EmailHistoryRepository, EmailMessage};
use local_store::DbPool;
use mail_parser::MessageParser;
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncReadCompatExt;
use tracing::{info, warn};
use uuid::Uuid;

/// Fetch emails from an IMAP server and store them in the database.
/// Returns (fetched, stored, skipped).
pub async fn fetch_emails(
    server: &str,
    port: u16,
    username: &str,
    password: &str,
    folder: &str,
    max_emails: u32,
    pool: &DbPool,
    user_id: Option<Uuid>,
) -> Result<(usize, usize, usize)> {
    info!("Connecting to IMAP server {}:{}", server, port);

    // Connect via TLS
    let tcp_stream = TcpStream::connect((server, port)).await?;
    let tls = TlsConnector::new();
    let tls_stream = tls.connect(server, tcp_stream.compat()).await?;

    let client = async_imap::Client::new(tls_stream);

    // Login
    let mut imap_session = client
        .login(username, password)
        .await
        .map_err(|e| anyhow::anyhow!("IMAP login failed: {:?}", e))?;

    // Select folder
    let mailbox = imap_session.select(folder).await?;
    let total_in_folder = mailbox.exists as u32;
    info!("Selected folder '{}': {} messages total", folder, total_in_folder);

    if total_in_folder == 0 {
        imap_session.logout().await?;
        return Ok((0, 0, 0));
    }

    // Fetch the most recent N messages by sequence number
    let start = if total_in_folder > max_emails {
        total_in_folder - max_emails + 1
    } else {
        1
    };
    let range = format!("{}:*", start);

    let messages_stream = imap_session.fetch(range, "RFC822").await?;
    let fetched_messages: Vec<_> = messages_stream.collect().await;

    info!("Fetched {} messages from IMAP", fetched_messages.len());

    let repo = EmailHistoryRepository::new(pool);
    let parser = MessageParser::default();
    let now = chrono::Utc::now();

    let mut parsed_emails: Vec<EmailMessage> = Vec::new();
    let mut fetch_count = 0;

    for msg_result in &fetched_messages {
        match msg_result {
            Ok(fetch) => {
                fetch_count += 1;
                let body = match fetch.body() {
                    Some(b) => b,
                    None => {
                        warn!("No body in fetched message, skipping");
                        continue;
                    }
                };

                let message = match parser.parse(body) {
                    Some(m) => m,
                    None => {
                        warn!("Failed to parse email, skipping");
                        continue;
                    }
                };

                let from_address = message
                    .from()
                    .and_then(|addrs| addrs.first())
                    .and_then(|addr| addr.address())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "unknown@unknown.com".to_string());

                let from_name = message
                    .from()
                    .and_then(|addrs| addrs.first())
                    .and_then(|addr| addr.name())
                    .map(|s| s.to_string());

                let to_address = message
                    .to()
                    .and_then(|addrs| addrs.first())
                    .and_then(|addr| addr.address())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| username.to_string());

                let cc = message
                    .cc()
                    .map(|addrs| {
                        addrs
                            .iter()
                            .filter_map(|a| a.address().map(|s| s.to_string()))
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .filter(|s| !s.is_empty());

                let subject = message
                    .subject()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "(No Subject)".to_string());

                let body_text = message.body_text(0).map(|s| s.to_string());
                let body_html = message.body_html(0).map(|s| s.to_string());

                let message_id = message.message_id().map(|s| s.to_string());
                let in_reply_to = message.in_reply_to().as_text().map(|s| s.to_string());

                // Determine message date from headers, fallback to now
                let message_date = message
                    .date()
                    .map(|d| d.to_timestamp() * 1000) // Convert seconds to milliseconds
                    .unwrap_or_else(|| now.timestamp_millis());

                // Determine if sent or received based on folder
                let message_type = if folder.contains("Sent") { 2 } else { 1 };

                let has_attachments = if message.attachment_count() > 0 { 1 } else { 0 };

                parsed_emails.push(EmailMessage {
                    id: Uuid::new_v4(),
                    contact_id: None, // Will be matched later if desired
                    from_address,
                    from_name,
                    to_address,
                    cc,
                    bcc: None,
                    subject,
                    body_text,
                    body_html,
                    message_date,
                    message_type,
                    message_id,
                    in_reply_to,
                    email_references: None,
                    read_status: 0,
                    has_attachments,
                    folder: Some(folder.to_string()),
                    imported_at: now,
                    imported_by: "imap_fetch".to_string(),
                    source_server: Some(server.to_string()),
                });
            }
            Err(e) => {
                warn!("Failed to fetch message: {}", e);
            }
        }
    }

    // Batch insert (INSERT OR IGNORE handles deduplication on message_id uniqueness)
    let stored = repo.batch_create(&parsed_emails).await.map_err(|e| {
        anyhow::anyhow!("Failed to store emails: {}", e)
    })?;

    let skipped = parsed_emails.len() - stored;

    info!(
        "Email import complete: {} fetched, {} stored, {} skipped (duplicates)",
        fetch_count, stored, skipped
    );

    // Auto-link imported emails to contacts
    if let Some(uid) = user_id {
        crate::contact_reconciliation::auto_link_after_email_import(pool, uid).await;
    }

    // Run concept detection (keyword + rule-based) on newly imported emails
    if stored > 0 {
        let detector = crate::concept_detection::ConceptDetector::new(pool);
        for email in &parsed_emails {
            let comm = crate::concept_detection::CommunicationFields::from_email(email);
            if let Err(e) = detector.full_scan(&comm).await {
                warn!("Concept detection failed for email {}: {}", email.id, e);
            }
        }
    }

    imap_session.logout().await?;

    Ok((fetch_count, stored, skipped))
}
