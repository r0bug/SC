use crate::auth::{login, signup, AuthConfig};
use crate::import;
use ai_middleware::{SegmindClient, SuggestionEngine};
use anyhow::Result;
use chrono::Utc;
use core_domain::*;
use import_service::ImportService;
use local_store::{
    CommunicationRepository, ContactRepository, LocalStore, NoteRepository, ShareRepository,
};
use std::fs;
use std::path::PathBuf;
use tracing::info;
use uuid::Uuid;

pub async fn import_command(
    csv: Option<String>,
    vcard: Option<String>,
    sms: Option<String>,
) -> Result<()> {
    let config = AuthConfig::load()?;

    // Check if authenticated for sync service import
    if config.is_authenticated() {
        println!("Using sync service for import (authenticated)");
        // Use sync service API for import
        if let Some(csv_path) = csv {
            let client = reqwest::Client::new();
            let file_contents = fs::read_to_string(&csv_path)?;
            let response = client
                .post(format!("{}/api/import/csv", config.api_url))
                .bearer_auth(config.token.as_ref().unwrap())
                .json(&serde_json::json!({
                    "data": file_contents,
                    "format": "csv"
                }))
                .send()
                .await?;

            if response.status().is_success() {
                println!("CSV import successful via sync service");
            } else {
                println!("Import failed: {}", response.status());
            }
        }
    } else {
        // Use local import service
        println!("Using local import (not authenticated)");
        let store = LocalStore::new("sqlite:./data/contacts.db").await?;
        let import_service = ImportService::new(store.pool().clone());

        if let Some(csv_path) = csv {
            info!("Importing from CSV: {}", csv_path);
            let path = PathBuf::from(csv_path);
            let result = import_service.import_file(&path, None, false).await?;

            if result.success {
                if let Some(summary) = result.summary {
                    println!("Import successful:");
                    println!("  Inserted: {}", summary.inserted);
                    println!("  Updated: {}", summary.updated);
                    println!("  Skipped: {}", summary.skipped);
                }
            } else {
                println!("Import failed - validation errors:");
                for error in result.validation.errors.iter().take(10) {
                    println!("  Row {}: {} - {}", error.row, error.field, error.message);
                }
            }
        }

        if let Some(vcard_path) = vcard {
            info!("Importing from vCard: {}", vcard_path);
            let path = PathBuf::from(vcard_path);
            let _result = import_service.import_file(&path, None, false).await?;
            println!("vCard import completed");
        }

        if let Some(sms_path) = sms {
            info!("Importing from SMS: {}", sms_path);
            import::import_sms(&sms_path, &store).await?;
        }
    }

    Ok(())
}

pub async fn list_command(limit: i64, only_named: bool) -> Result<()> {
    let config = AuthConfig::load()?;

    if config.is_authenticated() {
        // Use sync service API
        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/api/contacts?limit={}", config.api_url, limit))
            .bearer_auth(config.token.as_ref().unwrap())
            .send()
            .await?;

        if response.status().is_success() {
            let mut contacts: Vec<Contact> = response.json().await?;

            // Apply client-side filtering
            if only_named {
                contacts.retain(|c| !c.first_name.is_empty() || c.last_name.is_some());
            }

            println!("Contacts (from sync service):");
            for contact in contacts {
                println!(
                    "  {} {} - {} - {}",
                    contact.first_name,
                    contact.last_name.as_deref().unwrap_or(""),
                    contact.email.as_deref().unwrap_or("N/A"),
                    contact.phone.as_deref().unwrap_or("N/A")
                );
            }
        } else {
            println!("Failed to fetch contacts: {}", response.status());
        }
    } else {
        // Use local store
        let store = LocalStore::new("sqlite:./data/contacts.db").await?;
        let repo = ContactRepository::new(store.pool());

        let mut contacts = repo.list(limit, 0).await.map_err(|e| anyhow::anyhow!(e))?;

        // Apply filtering
        if only_named {
            contacts.retain(|c| !c.first_name.is_empty() || c.last_name.is_some());
        }

        println!("Contacts (local):");
        for contact in contacts {
            println!(
                "  {} {} - {} - {}",
                contact.first_name,
                contact.last_name.as_deref().unwrap_or(""),
                contact.email.as_deref().unwrap_or("N/A"),
                contact.phone.as_deref().unwrap_or("N/A")
            );
        }
    }

    Ok(())
}

pub async fn search_command(query: &str) -> Result<()> {
    let store = LocalStore::new("sqlite:./data/contacts.db").await?;
    let repo = ContactRepository::new(store.pool());

    let contacts = repo.search(query).await.map_err(|e| anyhow::anyhow!(e))?;

    println!("Found {} contacts:", contacts.len());
    for contact in contacts {
        println!(
            "  {} {} - {}",
            contact.first_name,
            contact.last_name.as_deref().unwrap_or(""),
            contact.email.as_deref().unwrap_or("N/A")
        );
    }

    Ok(())
}

pub async fn add_command(
    first_name: String,
    last_name: Option<String>,
    email: Option<String>,
    phone: Option<String>,
) -> Result<()> {
    let config = AuthConfig::load()?;

    if config.is_authenticated() {
        // Use sync service API
        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/api/contacts", config.api_url))
            .bearer_auth(config.token.as_ref().unwrap())
            .json(&serde_json::json!({
                "first_name": first_name,
                "last_name": last_name,
                "email": email,
                "phone": phone
            }))
            .send()
            .await?;

        if response.status().is_success() {
            let contact: Contact = response.json().await?;
            println!("Contact created via sync service: {}", contact.id);
        } else {
            println!("Failed to create contact: {}", response.status());
        }
    } else {
        // Use local store
        let store = LocalStore::new("sqlite:./data/contacts.db").await?;
        let repo = ContactRepository::new(store.pool());

        let placeholder_user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap();

        let contact = Contact {
            id: Uuid::new_v4(),
            first_name,
            last_name,
            email,
            phone,
            organization: None,
            title: None,
            notes: None,
            social_handles: vec![],
            tags: vec![],
            projects: vec![],
            groups: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: placeholder_user_id,
            version: 1,
            last_synced_at: None,
            metadata: serde_json::json!({}),
        };

        repo.create(&contact)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;

        println!("Contact created locally: {}", contact.id);
    }

    Ok(())
}

pub async fn note_command(contact_id: &str, title: String, content: String) -> Result<()> {
    let store = LocalStore::new("sqlite:./data/contacts.db").await?;
    let repo = NoteRepository::new(store.pool());

    let contact_uuid = Uuid::parse_str(contact_id)?;

    // Use a placeholder user ID for CLI operations
    let placeholder_user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap();

    let note = Note {
        id: Uuid::new_v4(),
        contact_id: Some(contact_uuid),
        project_id: None,
        title,
        content,
        attachment_ids: vec![],
        created_at: Utc::now(),
        updated_at: Utc::now(),
        created_by: placeholder_user_id,
        version: 1,
        last_synced_at: None,
    };

    repo.create(&note).await.map_err(|e| anyhow::anyhow!(e))?;

    println!("Note created: {}", note.id);
    Ok(())
}

pub async fn communicate_command(contact_id: &str, method: &str, message: String) -> Result<()> {
    let store = LocalStore::new("sqlite:./data/contacts.db").await?;
    let contact_repo = ContactRepository::new(store.pool());
    let comm_repo = CommunicationRepository::new(store.pool());

    let contact_uuid = Uuid::parse_str(contact_id)?;
    let contact = contact_repo
        .get_by_id(contact_uuid)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    let comm_method = match method {
        "email" => CommunicationMethod::Email,
        "sms" => CommunicationMethod::SMS,
        platform => CommunicationMethod::Social {
            platform: platform.to_string(),
        },
    };

    let attempt = CommunicationAttempt {
        id: Uuid::new_v4(),
        contact_id: contact_uuid,
        method: comm_method.clone(),
        subject: None,
        message: message.clone(),
        status: CommunicationStatus::Pending,
        scheduled_at: None,
        attempted_at: None,
        retry_count: 0,
        created_at: Utc::now(),
    };

    comm_repo
        .create(&attempt)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    println!("\n✅ Communication queued: {}", attempt.id);

    let method_display = match method {
        "email" => "EMAIL".to_string(),
        "sms" => "SMS".to_string(),
        platform => format!("{} message", platform.to_uppercase()),
    };

    println!(
        "\n⚠️  [MOCK] This is a SIMULATED communication - NO actual {} will be sent!",
        method_display
    );
    println!("\n📋 Details:");
    println!(
        "   Recipient: {} {}",
        contact.first_name,
        contact.last_name.as_deref().unwrap_or("")
    );

    match &comm_method {
        CommunicationMethod::Email => {
            if let Some(email) = &contact.email {
                println!("   Email: {}", email);
            } else {
                println!("   ⚠️  Warning: Contact has no email address");
            }
        }
        CommunicationMethod::SMS => {
            if let Some(phone) = &contact.phone {
                println!("   Phone: {}", phone);
            } else {
                println!("   ⚠️  Warning: Contact has no phone number");
            }
        }
        CommunicationMethod::Social { platform } => {
            println!("   Platform: {}", platform);
        }
    }

    println!(
        "   Message: {}",
        if message.len() > 50 {
            format!("{}...", &message[..50])
        } else {
            message
        }
    );

    println!("\n💡 Alpha Limitation:");
    println!("   All Email/SMS/Social sends are MOCKED in this release.");
    println!("   The communication has been logged to the database but will NOT be");
    println!("   delivered to any real service. This allows testing the workflow");
    println!("   without requiring actual SMTP/SMS credentials.");

    println!("\n🌐 Try the Web UI:");
    println!("   Visit http://localhost:3001/communications to use the placeholder");
    println!("   communication forms with explicit Email/SMS cards.");

    println!("\n📊 To view queued communications:");
    println!("   Check the database or run background worker to process mocks.");

    Ok(())
}

pub async fn share_command(entity_type: &str, entity_id: &str, email: &str) -> Result<()> {
    let store = LocalStore::new("sqlite:./data/contacts.db").await?;
    let repo = ShareRepository::new(store.pool());

    let entity_uuid = Uuid::parse_str(entity_id)?;
    let share_type = match entity_type {
        "contact" => ShareEntityType::Contact,
        "project" => ShareEntityType::Project,
        "note" => ShareEntityType::Note,
        _ => anyhow::bail!("Invalid entity type"),
    };

    // Use a placeholder user ID for CLI operations
    let placeholder_user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap();

    let invite = ShareInvite {
        id: Uuid::new_v4(),
        entity_type: share_type,
        entity_id: entity_uuid,
        shared_by: placeholder_user_id,
        shared_with_email: email.to_string(),
        shared_with_user: None,
        permissions: vec![Permission::Read],
        accepted: false,
        accepted_at: None,
        revoked: false,
        revoked_at: None,
        created_at: Utc::now(),
        expires_at: None,
    };

    repo.create(&invite).await.map_err(|e| anyhow::anyhow!(e))?;

    println!("Share invite created: {}", invite.id);
    Ok(())
}

pub async fn suggest_command(contact_id: &str) -> Result<()> {
    let store = LocalStore::new("sqlite:./data/contacts.db").await?;
    let contact_repo = ContactRepository::new(store.pool());

    let contact_uuid = Uuid::parse_str(contact_id)?;
    let contact = contact_repo
        .get_by_id(contact_uuid)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    let client = SegmindClient::new(Some("mock-api-key".to_string()));
    let engine = SuggestionEngine::new(client);

    let suggestions = engine.generate_contact_suggestions(&contact).await?;

    println!(
        "AI Suggestions for {} {}:",
        contact.first_name,
        contact.last_name.as_deref().unwrap_or("")
    );
    for suggestion in suggestions {
        println!("  [{}] {}", suggestion.confidence, suggestion.content);
    }

    Ok(())
}

// Authentication commands
pub async fn login_command(email: String, password: String) -> Result<()> {
    let mut config = AuthConfig::load()?;

    let response = login(email, password, &config.api_url).await?;

    config.token = Some(response.token);
    config.user_id = Some(response.user.id);
    config.email = Some(response.user.email);
    config.save()?;

    println!("Successfully logged in as {}", response.user.name);
    Ok(())
}

pub async fn signup_command(email: String, password: String, name: String) -> Result<()> {
    let mut config = AuthConfig::load()?;

    let response = signup(email, password, name, &config.api_url).await?;

    config.token = Some(response.token);
    config.user_id = Some(response.user.id);
    config.email = Some(response.user.email);
    config.save()?;

    println!(
        "Successfully signed up and logged in as {}",
        response.user.name
    );
    Ok(())
}

pub async fn logout_command() -> Result<()> {
    let mut config = AuthConfig::load()?;
    config.clear();
    config.save()?;

    println!("Successfully logged out");
    Ok(())
}

pub async fn status_command() -> Result<()> {
    let config = AuthConfig::load()?;

    if config.is_authenticated() {
        println!(
            "Logged in as: {}",
            config.email.as_deref().unwrap_or("unknown")
        );
        println!("API URL: {}", config.api_url);
    } else {
        println!("Not logged in");
    }

    Ok(())
}

pub async fn merge_duplicates_command(dry_run: bool, phone: Option<String>) -> Result<()> {
    use std::io::{self, Write};

    let store = LocalStore::new("sqlite:./data/contacts.db").await?;
    let contact_repo = ContactRepository::new(store.pool());

    println!("Scanning for duplicate contacts by phone number...\n");

    let duplicates = contact_repo.find_duplicates_by_phone().await?;

    if duplicates.is_empty() {
        println!("No duplicate contacts found!");
        return Ok(());
    }

    // Filter by phone if specified
    let filtered_duplicates: Vec<_> = if let Some(filter_phone) = &phone {
        duplicates
            .into_iter()
            .filter(|(p, _)| p == filter_phone)
            .collect()
    } else {
        duplicates.into_iter().collect()
    };

    if filtered_duplicates.is_empty() {
        println!("No duplicate contacts found for phone: {}", phone.unwrap());
        return Ok(());
    }

    println!("Found {} phone numbers with duplicates:\n", filtered_duplicates.len());

    let mut total_merges = 0;
    let mut total_duplicates_removed = 0;

    for (phone_num, contacts) in filtered_duplicates {
        println!("Phone: {}", phone_num);
        println!("  {} duplicate contacts found:", contacts.len());

        for (idx, contact) in contacts.iter().enumerate() {
            let name = format!("{} {}",
                contact.first_name,
                contact.last_name.as_deref().unwrap_or("")
            ).trim().to_string();

            println!("    {}. {} (ID: {})", idx + 1, name, contact.id);
            if let Some(email) = &contact.email {
                println!("       Email: {}", email);
            }
            if contact.notes.is_some() {
                println!("       Has notes: Yes");
            }
            if !contact.tags.is_empty() {
                println!("       Tags: {}", contact.tags.len());
            }
            if !contact.projects.is_empty() {
                println!("       Projects: {}", contact.projects.len());
            }
        }

        if dry_run {
            println!("  [DRY RUN] Would merge {} duplicates into the oldest contact\n", contacts.len() - 1);
            total_duplicates_removed += contacts.len() - 1;
        } else {
            // Ask user which contact to keep (default: oldest = first one)
            print!("  Which contact to keep? [1-{}] (default: 1 - oldest): ", contacts.len());
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            let keep_idx = if input.is_empty() {
                0
            } else {
                match input.parse::<usize>() {
                    Ok(idx) if idx > 0 && idx <= contacts.len() => idx - 1,
                    _ => {
                        println!("  Invalid selection, skipping this group\n");
                        continue;
                    }
                }
            };

            let keep_contact = &contacts[keep_idx];
            println!("  Keeping: {} (ID: {})",
                format!("{} {}", keep_contact.first_name, keep_contact.last_name.as_deref().unwrap_or("")).trim(),
                keep_contact.id
            );

            // Merge all other contacts into the kept one
            for (idx, contact) in contacts.iter().enumerate() {
                if idx != keep_idx {
                    print!("  Merging {} into kept contact... ",
                        format!("{} {}", contact.first_name, contact.last_name.as_deref().unwrap_or("")).trim()
                    );
                    io::stdout().flush()?;

                    match contact_repo.merge_contacts(keep_contact.id, contact.id).await {
                        Ok(stats) => {
                            println!("Done!");
                            println!("    SMS messages: {}", stats.sms_messages_relinked);
                            println!("    Communications: {}", stats.communications_relinked);
                            println!("    Notes: {}", stats.notes_relinked);
                            println!("    Social handles: {}", stats.social_handles_merged);
                            println!("    Tags: {}", stats.tags_merged);
                            println!("    Projects: {}", stats.projects_merged);
                            println!("    Groups: {}", stats.groups_merged);
                            total_merges += 1;
                            total_duplicates_removed += 1;
                        }
                        Err(e) => {
                            println!("Failed: {}", e);
                        }
                    }
                }
            }
            println!();
        }
    }

    if dry_run {
        println!("\n[DRY RUN] Summary:");
        println!("  Would remove {} duplicate contacts", total_duplicates_removed);
        println!("\nRun without --dry-run to actually perform the merges");
    } else {
        println!("\nMerge complete!");
        println!("  Performed {} merge operations", total_merges);
        println!("  Removed {} duplicate contacts", total_duplicates_removed);
    }

    Ok(())
}

// Re-export extended commands
pub use crate::commands_extended::*;
