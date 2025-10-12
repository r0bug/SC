use anyhow::Result;
use import_service::connectors::{GoogleContactsImporter, StreamingAndroidParser};
use local_store::{ContactRepository, LocalStore};
use std::io::{self, Write};
use std::path::Path;
use tracing::info;

/// Import Google Contacts CSV file
pub async fn import_google_contacts_csv(path: &str, store: &LocalStore) -> Result<()> {
    println!("\n📱 Google Contacts CSV Import");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("File: {}", path);

    let contacts = GoogleContactsImporter::parse_csv(Path::new(path))?;

    println!("\n✅ Parsed {} contacts", contacts.len());
    println!("\n📊 Preview (first 5):");
    for (idx, contact) in contacts.iter().take(5).enumerate() {
        println!(
            "  {}. {} {} - {}",
            idx + 1,
            contact.first_name,
            contact.last_name.as_deref().unwrap_or(""),
            contact
                .phone
                .as_ref()
                .or(contact.email.as_ref())
                .unwrap_or(&"(no contact info)".to_string())
        );
    }

    print!("\n❓ Import {} contacts? [y/N]: ", contacts.len());
    io::stdout().flush()?;
    let mut response = String::new();
    io::stdin().read_line(&mut response)?;

    if response.trim().to_lowercase() != "y" {
        println!("❌ Import cancelled");
        return Ok(());
    }

    println!("\n⏳ Importing contacts...");
    let repo = ContactRepository::new(store.pool());
    let mut imported = 0;
    let mut failed = 0;
    let total_count = contacts.len();

    for contact in contacts {
        match repo.create(&contact).await {
            Ok(_) => {
                imported += 1;
                if imported % 100 == 0 {
                    print!("\r   Progress: {}/{}", imported, total_count);
                    io::stdout().flush()?;
                }
            }
            Err(e) => {
                eprintln!("\n⚠️  Failed to import {}: {}", contact.first_name, e);
                failed += 1;
            }
        }
    }

    println!("\n\n✅ Import complete!");
    println!("   Imported: {}", imported);
    if failed > 0 {
        println!("   Failed: {}", failed);
    }

    Ok(())
}

/// Import large SMS XML file using streaming parser
pub async fn import_sms_xml_streaming(path: &str, store: &LocalStore) -> Result<()> {
    println!("\n💬 SMS XML Streaming Import");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("File: {}", path);
    println!("⚠️  Large file detected - using streaming parser");
    println!();

    print!("❓ This will import SMS messages and create contacts. Continue? [y/N]: ");
    io::stdout().flush()?;
    let mut response = String::new();
    io::stdin().read_line(&mut response)?;

    if response.trim().to_lowercase() != "y" {
        println!("❌ Import cancelled");
        return Ok(());
    }

    println!("\n⏳ Starting streaming import...");
    println!("   Processing in batches of 1000...\n");

    let repo = ContactRepository::new(store.pool());
    let parser = StreamingAndroidParser::new(1000);

    let stats = parser.parse_sms_stream(Path::new(path), |batch| {
        let repo = ContactRepository::new(store.pool());
        async move {
            info!("Processing batch of {} contacts", batch.len());

            let mut batch_imported = 0;
            let mut batch_failed = 0;

            for contact in batch {
                match repo.create(&contact).await {
                    Ok(_) => batch_imported += 1,
                    Err(e) => {
                        // Skip duplicates silently
                        if !e.to_string().contains("UNIQUE") {
                            eprintln!("⚠️  Failed to import contact: {}", e);
                        }
                        batch_failed += 1;
                    }
                }
            }

            println!(
                "   Batch complete: {} imported in this batch",
                batch_imported
            );

            Ok(())
        }
    }).await?;

    println!("\n✅ Streaming import complete!");
    println!("   Messages processed: {}", stats.total_messages);
    println!("   Unique contacts found: {}", stats.contacts_created);
    println!("   Skipped: {}", stats.skipped);

    Ok(())
}

/// Import calls XML file using streaming parser
pub async fn import_calls_xml_streaming(path: &str, store: &LocalStore) -> Result<()> {
    println!("\n📞 Call History XML Streaming Import");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("File: {}", path);

    print!("❓ Import call history? [y/N]: ");
    io::stdout().flush()?;
    let mut response = String::new();
    io::stdin().read_line(&mut response)?;

    if response.trim().to_lowercase() != "y" {
        println!("❌ Import cancelled");
        return Ok(());
    }

    println!("\n⏳ Starting streaming import...");

    let repo = ContactRepository::new(store.pool());
    let parser = StreamingAndroidParser::new(1000);

    let stats = parser.parse_calls_stream(Path::new(path), |batch| {
        let repo = ContactRepository::new(store.pool());
        async move {
            let mut batch_imported = 0;
            let mut batch_failed = 0;

            for contact in batch {
                match repo.create(&contact).await {
                    Ok(_) => batch_imported += 1,
                    Err(e) => {
                        if !e.to_string().contains("UNIQUE") {
                            eprintln!("⚠️  Failed to import contact: {}", e);
                        }
                        batch_failed += 1;
                    }
                }
            }

            println!(
                "   Batch: {} imported in this batch",
                batch_imported
            );

            Ok(())
        }
    }).await?;

    println!("\n✅ Import complete!");
    println!("   Calls processed: {}", stats.total_messages);
    println!("   Unique contacts found: {}", stats.contacts_created);
    println!("   Skipped: {}", stats.skipped);

    Ok(())
}
