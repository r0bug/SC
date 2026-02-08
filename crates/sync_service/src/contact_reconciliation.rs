use core_domain::Contact;
use local_store::repositories::{ContactRepository, EmailHistoryRepository, SmsHistoryRepository};
use local_store::DbPool;
use serde::Serialize;
use tracing::{info, warn};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct ReconciliationResult {
    pub contacts_created: usize,
    pub contacts_matched: usize,
    pub email_messages_linked: u64,
    pub sms_messages_linked: u64,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UnlinkedStats {
    pub unlinked_email_senders: usize,
    pub unlinked_sms_senders: usize,
    pub total_unlinked_senders: usize,
}

/// Run full reconciliation across email and SMS history.
pub async fn reconcile_all(pool: &DbPool, user_id: Uuid) -> ReconciliationResult {
    let mut result = ReconciliationResult {
        contacts_created: 0,
        contacts_matched: 0,
        email_messages_linked: 0,
        sms_messages_linked: 0,
        errors: Vec::new(),
    };

    let email_result = reconcile_emails(pool, user_id).await;
    result.contacts_created += email_result.contacts_created;
    result.contacts_matched += email_result.contacts_matched;
    result.email_messages_linked += email_result.email_messages_linked;
    result.errors.extend(email_result.errors);

    let sms_result = reconcile_sms(pool, user_id).await;
    result.contacts_created += sms_result.contacts_created;
    result.contacts_matched += sms_result.contacts_matched;
    result.sms_messages_linked += sms_result.sms_messages_linked;
    result.errors.extend(sms_result.errors);

    info!(
        "Reconciliation complete: {} created, {} matched, {} emails linked, {} SMS linked, {} errors",
        result.contacts_created,
        result.contacts_matched,
        result.email_messages_linked,
        result.sms_messages_linked,
        result.errors.len()
    );

    result
}

/// Reconcile unlinked email senders: match existing contacts or create new ones.
pub async fn reconcile_emails(pool: &DbPool, user_id: Uuid) -> ReconciliationResult {
    let mut result = ReconciliationResult {
        contacts_created: 0,
        contacts_matched: 0,
        email_messages_linked: 0,
        sms_messages_linked: 0,
        errors: Vec::new(),
    };

    let email_repo = EmailHistoryRepository::new(pool);
    let contact_repo = ContactRepository::new(pool);

    let unlinked = match email_repo.get_unlinked_senders().await {
        Ok(u) => u,
        Err(e) => {
            result
                .errors
                .push(format!("Failed to get unlinked email senders: {}", e));
            return result;
        }
    };

    for sender in unlinked {
        // Try to find existing contact by email
        match contact_repo.get_by_email(&sender.identifier).await {
            Ok(Some(contact)) => {
                // Contact exists, link emails
                match email_repo
                    .link_emails_to_contact(&sender.identifier, contact.id)
                    .await
                {
                    Ok(linked) => {
                        result.contacts_matched += 1;
                        result.email_messages_linked += linked;
                    }
                    Err(e) => {
                        result.errors.push(format!(
                            "Failed to link emails for {}: {}",
                            sender.identifier, e
                        ));
                    }
                }
            }
            Ok(None) => {
                // No existing contact, create one
                let (first_name, last_name) =
                    parse_name_from_email(&sender.display_name, &sender.identifier);

                let now = chrono::Utc::now();
                let new_contact = Contact {
                    id: Uuid::new_v4(),
                    first_name,
                    last_name,
                    email: Some(sender.identifier.clone()),
                    phone: None,
                    organization: None,
                    title: None,
                    notes: Some("Auto-created from communication import".to_string()),
                    social_handles: Vec::new(),
                    tags: Vec::new(),
                    projects: Vec::new(),
                    groups: Vec::new(),
                    created_at: now,
                    updated_at: now,
                    created_by: user_id,
                    version: 1,
                    last_synced_at: None,
                    metadata: serde_json::json!({
                        "auto_created": true,
                        "source": "reconciliation"
                    }),
                };

                match contact_repo.create(&new_contact).await {
                    Ok(()) => {
                        result.contacts_created += 1;
                        match email_repo
                            .link_emails_to_contact(&sender.identifier, new_contact.id)
                            .await
                        {
                            Ok(linked) => {
                                result.email_messages_linked += linked;
                            }
                            Err(e) => {
                                result.errors.push(format!(
                                    "Failed to link emails for new contact {}: {}",
                                    sender.identifier, e
                                ));
                            }
                        }
                    }
                    Err(e) => {
                        result.errors.push(format!(
                            "Failed to create contact for {}: {}",
                            sender.identifier, e
                        ));
                    }
                }
            }
            Err(e) => {
                result.errors.push(format!(
                    "Failed to lookup contact for {}: {}",
                    sender.identifier, e
                ));
            }
        }
    }

    result
}

/// Reconcile unlinked SMS senders: match existing contacts or create new ones.
pub async fn reconcile_sms(pool: &DbPool, user_id: Uuid) -> ReconciliationResult {
    let mut result = ReconciliationResult {
        contacts_created: 0,
        contacts_matched: 0,
        email_messages_linked: 0,
        sms_messages_linked: 0,
        errors: Vec::new(),
    };

    let sms_repo = SmsHistoryRepository::new(pool);
    let contact_repo = ContactRepository::new(pool);

    let unlinked = match sms_repo.get_unlinked_senders().await {
        Ok(u) => u,
        Err(e) => {
            result
                .errors
                .push(format!("Failed to get unlinked SMS senders: {}", e));
            return result;
        }
    };

    for sender in unlinked {
        // Try to find existing contact by phone
        match contact_repo.get_by_phone(&sender.phone_number).await {
            Ok(Some(contact)) => {
                // Contact exists, link messages
                match sms_repo
                    .link_messages_to_contact(&sender.phone_number, contact.id)
                    .await
                {
                    Ok(linked) => {
                        result.contacts_matched += 1;
                        result.sms_messages_linked += linked;
                    }
                    Err(e) => {
                        result.errors.push(format!(
                            "Failed to link SMS for {}: {}",
                            sender.phone_number, e
                        ));
                    }
                }
            }
            Ok(None) => {
                // No existing contact, create one
                let first_name = sender
                    .contact_name
                    .clone()
                    .unwrap_or_else(|| sender.phone_number.clone());

                let now = chrono::Utc::now();
                let new_contact = Contact {
                    id: Uuid::new_v4(),
                    first_name,
                    last_name: None,
                    email: None,
                    phone: Some(sender.phone_number.clone()),
                    organization: None,
                    title: None,
                    notes: Some("Auto-created from communication import".to_string()),
                    social_handles: Vec::new(),
                    tags: Vec::new(),
                    projects: Vec::new(),
                    groups: Vec::new(),
                    created_at: now,
                    updated_at: now,
                    created_by: user_id,
                    version: 1,
                    last_synced_at: None,
                    metadata: serde_json::json!({
                        "auto_created": true,
                        "source": "reconciliation"
                    }),
                };

                match contact_repo.create(&new_contact).await {
                    Ok(()) => {
                        result.contacts_created += 1;
                        match sms_repo
                            .link_messages_to_contact(&sender.phone_number, new_contact.id)
                            .await
                        {
                            Ok(linked) => {
                                result.sms_messages_linked += linked;
                            }
                            Err(e) => {
                                result.errors.push(format!(
                                    "Failed to link SMS for new contact {}: {}",
                                    sender.phone_number, e
                                ));
                            }
                        }
                    }
                    Err(e) => {
                        result.errors.push(format!(
                            "Failed to create contact for {}: {}",
                            sender.phone_number, e
                        ));
                    }
                }
            }
            Err(e) => {
                result.errors.push(format!(
                    "Failed to lookup contact for {}: {}",
                    sender.phone_number, e
                ));
            }
        }
    }

    result
}

/// Get counts of unlinked communications for UI display.
pub async fn get_unlinked_stats(pool: &DbPool) -> UnlinkedStats {
    let email_repo = EmailHistoryRepository::new(pool);
    let sms_repo = SmsHistoryRepository::new(pool);

    let email_count = email_repo
        .get_unlinked_senders()
        .await
        .map(|v| v.len())
        .unwrap_or(0);

    let sms_count = sms_repo
        .get_unlinked_senders()
        .await
        .map(|v| v.len())
        .unwrap_or(0);

    UnlinkedStats {
        unlinked_email_senders: email_count,
        unlinked_sms_senders: sms_count,
        total_unlinked_senders: email_count + sms_count,
    }
}

/// Auto-link contacts after an email import. Best-effort, logs errors.
pub async fn auto_link_after_email_import(pool: &DbPool, user_id: Uuid) {
    info!("Auto-linking contacts after email import...");
    let result = reconcile_emails(pool, user_id).await;
    if !result.errors.is_empty() {
        warn!(
            "Email auto-link completed with {} errors: {:?}",
            result.errors.len(),
            result.errors
        );
    }
    info!(
        "Email auto-link: {} created, {} matched, {} messages linked",
        result.contacts_created, result.contacts_matched, result.email_messages_linked
    );
}

/// Auto-link contacts after an SMS/call import. Best-effort, logs errors.
pub async fn auto_link_after_sms_import(pool: &DbPool, user_id: Uuid) {
    info!("Auto-linking contacts after SMS import...");
    let result = reconcile_sms(pool, user_id).await;
    if !result.errors.is_empty() {
        warn!(
            "SMS auto-link completed with {} errors: {:?}",
            result.errors.len(),
            result.errors
        );
    }
    info!(
        "SMS auto-link: {} created, {} matched, {} messages linked",
        result.contacts_created, result.contacts_matched, result.sms_messages_linked
    );
}

/// Parse a display name or email address into (first_name, last_name).
fn parse_name_from_email(display_name: &Option<String>, email: &str) -> (String, Option<String>) {
    // Try display name first (e.g., "John Doe")
    if let Some(name) = display_name {
        let trimmed = name.trim();
        if !trimmed.is_empty() {
            let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();
            return if parts.len() >= 2 {
                (parts[0].to_string(), Some(parts[1].to_string()))
            } else {
                (parts[0].to_string(), None)
            };
        }
    }

    // Fallback: parse email local-part (e.g., "john.doe@example.com" -> "John", "Doe")
    let local_part = email.split('@').next().unwrap_or(email);
    let parts: Vec<&str> = local_part.split(['.', '_', '-']).collect();

    if parts.len() >= 2 {
        (capitalize(parts[0]), Some(capitalize(parts[1])))
    } else {
        (capitalize(parts[0]), None)
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().to_string() + c.as_str(),
    }
}
