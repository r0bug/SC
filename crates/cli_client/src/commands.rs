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

pub async fn list_command(limit: i64) -> Result<()> {
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
            let contacts: Vec<Contact> = response.json().await?;
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

        let contacts = repo.list(limit, 0).await.map_err(|e| anyhow::anyhow!(e))?;

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

// Re-export extended commands
pub use crate::commands_extended::*;
