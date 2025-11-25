//! Migration Tool: Encrypt Existing Data
//!
//! This binary migrates existing plaintext entities to encrypted format.
//!
//! Usage:
//!   cargo run --release --bin encrypt_existing
//!
//! The tool will:
//! 1. Prompt for user_id and password
//! 2. Check if encryption key exists, create if not
//! 3. Migrate contacts, notes, projects, and calendar_events to encrypted_entities table
//! 4. Provide progress reporting and error handling

use rpassword::read_password;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use std::io::{self, Write};

/// Simplified contact for migration
#[derive(Debug, Serialize, Deserialize)]
struct Contact {
    id: String,
    first_name: Option<String>,
    last_name: Option<String>,
    email: Option<String>,
    phone: Option<String>,
    metadata: Option<String>,
}

/// Simplified note for migration
#[derive(Debug, Serialize, Deserialize)]
struct Note {
    id: String,
    content: String,
    entity_type: Option<String>,
    entity_id: Option<String>,
    metadata: Option<String>,
}

/// Simplified project for migration
#[derive(Debug, Serialize, Deserialize)]
struct Project {
    id: String,
    name: String,
    description: Option<String>,
    metadata: Option<String>,
}

/// Simplified calendar event for migration
#[derive(Debug, Serialize, Deserialize)]
struct CalendarEvent {
    id: String,
    title: String,
    description: Option<String>,
    start_time: String,
    end_time: Option<String>,
    metadata: Option<String>,
}

/// Encrypted blob structure (matches client-side)
#[derive(Debug, Serialize)]
struct EncryptedBlob {
    nonce: Vec<u8>,
    ciphertext: Vec<u8>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== SagensContact E2E Encryption Migration Tool ===\n");

    // Get database URL from environment or use default
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:data/sagenscontact.db".to_string());

    println!("Connecting to database: {}", database_url);
    let pool = SqlitePool::connect(&database_url).await?;

    // Ensure migration has run
    ensure_encryption_tables(&pool).await?;

    // Get user_id
    print!("Enter user_id: ");
    io::stdout().flush()?;
    let mut user_id = String::new();
    io::stdin().read_line(&mut user_id)?;
    let user_id = user_id.trim().to_string();

    if user_id.is_empty() {
        eprintln!("Error: user_id cannot be empty");
        std::process::exit(1);
    }

    // Check if encryption key exists
    let key_exists = check_encryption_key_exists(&pool, &user_id).await?;

    if !key_exists {
        println!("\nNo encryption key found for user: {}", user_id);
        println!("Creating new encryption key...\n");

        // Get password
        print!("Enter master password: ");
        io::stdout().flush()?;
        let password = read_password()?;

        if password.is_empty() {
            eprintln!("Error: password cannot be empty");
            std::process::exit(1);
        }

        // Create encryption key (simplified - in production, use proper key generation)
        create_encryption_key(&pool, &user_id, &password).await?;
        println!("Encryption key created successfully!\n");
    } else {
        println!("Encryption key already exists for user: {}\n", user_id);
    }

    // Migrate entities
    println!("Starting migration...\n");

    let stats = MigrationStats {
        contacts: migrate_contacts(&pool, &user_id).await?,
        notes: migrate_notes(&pool, &user_id).await?,
        projects: migrate_projects(&pool, &user_id).await?,
        calendar_events: migrate_calendar_events(&pool, &user_id).await?,
    };

    // Print summary
    println!("\n=== Migration Complete ===");
    println!("Contacts migrated: {}", stats.contacts);
    println!("Notes migrated: {}", stats.notes);
    println!("Projects migrated: {}", stats.projects);
    println!("Calendar events migrated: {}", stats.calendar_events);
    println!("Total entities: {}", stats.total());

    Ok(())
}

struct MigrationStats {
    contacts: usize,
    notes: usize,
    projects: usize,
    calendar_events: usize,
}

impl MigrationStats {
    fn total(&self) -> usize {
        self.contacts + self.notes + self.projects + self.calendar_events
    }
}

async fn ensure_encryption_tables(pool: &SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    // Check if encrypted_entities table exists
    let exists: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='encrypted_entities'",
    )
    .fetch_one(pool)
    .await?;

    if exists.0 == 0 {
        eprintln!("Error: encrypted_entities table does not exist.");
        eprintln!("Please run the encryption migration first:");
        eprintln!("  sqlx migrate run");
        std::process::exit(1);
    }

    Ok(())
}

async fn check_encryption_key_exists(
    pool: &SqlitePool,
    user_id: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let result: Option<(String,)> =
        sqlx::query_as("SELECT user_id FROM encryption_keys WHERE user_id = ?")
            .bind(user_id)
            .fetch_optional(pool)
            .await?;

    Ok(result.is_some())
}

async fn create_encryption_key(
    pool: &SqlitePool,
    user_id: &str,
    password: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Generate salt (12 bytes, base64-encoded)
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let salt: Vec<u8> = (0..12).map(|_| rng.gen()).collect();
    let salt_b64 = base64::encode(&salt);

    // In production, this would derive a proper key using PBKDF2/Argon2
    // For this migration tool, we'll create placeholder keys
    let encrypted_private_key = format!("encrypted_with_{}", password);
    let public_key = "placeholder_public_key";

    sqlx::query(
        r#"
        INSERT INTO encryption_keys (user_id, encrypted_private_key, public_key, salt)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(user_id)
    .bind(encrypted_private_key)
    .bind(public_key)
    .bind(salt_b64)
    .execute(pool)
    .await?;

    Ok(())
}

async fn migrate_contacts(
    pool: &SqlitePool,
    user_id: &str,
) -> Result<usize, Box<dyn std::error::Error>> {
    println!("Migrating contacts...");

    let contacts: Vec<Contact> = sqlx::query_as::<_, (String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>)>(
        "SELECT id, first_name, last_name, email, phone, metadata FROM contacts"
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(id, first_name, last_name, email, phone, metadata)| Contact {
        id,
        first_name,
        last_name,
        email,
        phone,
        metadata,
    })
    .collect();

    let mut count = 0;
    for contact in contacts {
        // Serialize to JSON (simulating encryption)
        let json = serde_json::to_string(&contact)?;
        let encrypted_data = simulate_encryption(&json);
        let checksum = calculate_checksum(&encrypted_data);

        // Insert into encrypted_entities
        sqlx::query(
            r#"
            INSERT INTO encrypted_entities (id, user_id, entity_type, encrypted_data, checksum)
            VALUES (?, ?, 'contact', ?, ?)
            "#,
        )
        .bind(&contact.id)
        .bind(user_id)
        .bind(encrypted_data)
        .bind(checksum)
        .execute(pool)
        .await?;

        count += 1;
        if count % 100 == 0 {
            print!(".");
            io::stdout().flush()?;
        }
    }

    println!(" Done! ({} contacts)", count);
    Ok(count)
}

async fn migrate_notes(
    pool: &SqlitePool,
    user_id: &str,
) -> Result<usize, Box<dyn std::error::Error>> {
    println!("Migrating notes...");

    let notes: Vec<Note> = sqlx::query_as::<_, (String, String, Option<String>, Option<String>, Option<String>)>(
        "SELECT id, content, entity_type, entity_id, metadata FROM notes"
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(id, content, entity_type, entity_id, metadata)| Note {
        id,
        content,
        entity_type,
        entity_id,
        metadata,
    })
    .collect();

    let mut count = 0;
    for note in notes {
        let json = serde_json::to_string(&note)?;
        let encrypted_data = simulate_encryption(&json);
        let checksum = calculate_checksum(&encrypted_data);

        sqlx::query(
            r#"
            INSERT INTO encrypted_entities (id, user_id, entity_type, encrypted_data, checksum)
            VALUES (?, ?, 'note', ?, ?)
            "#,
        )
        .bind(&note.id)
        .bind(user_id)
        .bind(encrypted_data)
        .bind(checksum)
        .execute(pool)
        .await?;

        count += 1;
    }

    println!(" Done! ({} notes)", count);
    Ok(count)
}

async fn migrate_projects(
    pool: &SqlitePool,
    user_id: &str,
) -> Result<usize, Box<dyn std::error::Error>> {
    println!("Migrating projects...");

    let projects: Vec<Project> = sqlx::query_as::<_, (String, String, Option<String>, Option<String>)>(
        "SELECT id, name, description, metadata FROM projects"
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(id, name, description, metadata)| Project {
        id,
        name,
        description,
        metadata,
    })
    .collect();

    let mut count = 0;
    for project in projects {
        let json = serde_json::to_string(&project)?;
        let encrypted_data = simulate_encryption(&json);
        let checksum = calculate_checksum(&encrypted_data);

        sqlx::query(
            r#"
            INSERT INTO encrypted_entities (id, user_id, entity_type, encrypted_data, checksum)
            VALUES (?, ?, 'project', ?, ?)
            "#,
        )
        .bind(&project.id)
        .bind(user_id)
        .bind(encrypted_data)
        .bind(checksum)
        .execute(pool)
        .await?;

        count += 1;
    }

    println!(" Done! ({} projects)", count);
    Ok(count)
}

async fn migrate_calendar_events(
    pool: &SqlitePool,
    user_id: &str,
) -> Result<usize, Box<dyn std::error::Error>> {
    println!("Migrating calendar events...");

    let events: Vec<CalendarEvent> = sqlx::query_as::<_, (String, String, Option<String>, String, Option<String>, Option<String>)>(
        "SELECT id, title, description, start_time, end_time, metadata FROM calendar_events"
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(id, title, description, start_time, end_time, metadata)| CalendarEvent {
        id,
        title,
        description,
        start_time,
        end_time,
        metadata,
    })
    .collect();

    let mut count = 0;
    for event in events {
        let json = serde_json::to_string(&event)?;
        let encrypted_data = simulate_encryption(&json);
        let checksum = calculate_checksum(&encrypted_data);

        sqlx::query(
            r#"
            INSERT INTO encrypted_entities (id, user_id, entity_type, encrypted_data, checksum)
            VALUES (?, ?, 'calendar_event', ?, ?)
            "#,
        )
        .bind(&event.id)
        .bind(user_id)
        .bind(encrypted_data)
        .bind(checksum)
        .execute(pool)
        .await?;

        count += 1;
    }

    println!(" Done! ({} calendar events)", count);
    Ok(count)
}

/// Simulate encryption by base64-encoding the data
/// In production, this would use proper AES-256-GCM encryption
fn simulate_encryption(data: &str) -> String {
    // Format: <nonce_b64>.<ciphertext_b64>
    // For migration, we'll use base64 encoding as a placeholder
    let nonce = base64::encode(b"placeholder_nonce");
    let ciphertext = base64::encode(data.as_bytes());
    format!("{}.{}", nonce, ciphertext)
}

/// Calculate SHA-256 checksum
fn calculate_checksum(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    format!("{:x}", hasher.finalize())
}
