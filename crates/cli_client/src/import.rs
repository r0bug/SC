use anyhow::Result;
use core_domain::*;
use local_store::{LocalStore, ContactRepository};
use uuid::Uuid;
use chrono::{Utc, DateTime};
use std::fs::File;
use csv::Reader;
use tracing::{info, warn};
use std::io::{self, Write, BufReader};
use std::collections::HashMap;
use quick_xml::Reader as XmlReader;
use quick_xml::events::Event;

pub async fn import_csv(path: &str, store: &LocalStore) -> Result<()> {
    let file = File::open(path)?;
    let mut reader = Reader::from_reader(file);

    let headers = reader.headers()?.clone();
    println!("\n📋 CSV Import Preview");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("File: {}", path);
    println!("Detected columns: {:?}", headers.iter().collect::<Vec<_>>());
    println!();

    let field_map = build_field_mapping(&headers)?;

    let mut records: Vec<csv::StringRecord> = vec![];
    let mut errors = vec![];

    for (idx, result) in reader.records().enumerate() {
        match result {
            Ok(record) => records.push(record),
            Err(e) => errors.push(format!("Row {}: {}", idx + 2, e))
        }
    }

    println!("✅ Parsed {} valid records", records.len());
    if !errors.is_empty() {
        warn!("⚠️  {} rows had parse errors:", errors.len());
        for err in &errors {
            warn!("  - {}", err);
        }
    }

    if !records.is_empty() {
        println!("\n📊 Preview (first 3 rows):");
        for (idx, record) in records.iter().take(3).enumerate() {
            println!("\n  Row {}:", idx + 1);
            for (col_idx, value) in record.iter().enumerate() {
                if let Some(header) = headers.get(col_idx) {
                    if let Some(_field) = field_map.get(header) {
                        println!("    {} → {}", header, value);
                    }
                }
            }
        }
    }

    print!("\n❓ Import {} contacts? [y/N]: ", records.len());
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

    for record in records {
        match parse_csv_record(&record, &headers, &field_map) {
            Ok(contact) => {
                match repo.create(&contact).await {
                    Ok(_) => {
                        info!("Imported: {} {}", contact.first_name, contact.last_name.as_deref().unwrap_or(""));
                        imported += 1;
                    }
                    Err(e) => {
                        warn!("Failed to save contact: {}", e);
                        failed += 1;
                    }
                }
            }
            Err(e) => {
                warn!("Failed to parse record: {}", e);
                failed += 1;
            }
        }
    }

    println!("\n✅ Import complete!");
    println!("   Imported: {}", imported);
    if failed > 0 {
        println!("   Failed: {}", failed);
    }

    Ok(())
}

fn build_field_mapping(headers: &csv::StringRecord) -> Result<HashMap<String, String>> {
    let mut map = HashMap::new();

    for header in headers.iter() {
        let normalized = header.trim().to_lowercase();
        let field = if normalized.contains("first") && normalized.contains("name") {
            "first_name"
        } else if normalized.contains("last") && normalized.contains("name") {
            "last_name"
        } else if normalized.contains("email") {
            "email"
        } else if normalized.contains("phone") || normalized.contains("mobile") {
            "phone"
        } else if normalized.contains("org") || normalized.contains("company") {
            "organization"
        } else if normalized.contains("title") || normalized.contains("position") {
            "title"
        } else {
            continue;
        };

        map.insert(header.to_string(), field.to_string());
    }

    if !map.values().any(|v| v == "first_name") {
        anyhow::bail!("CSV must have a 'first_name' or 'First Name' column");
    }

    Ok(map)
}

fn parse_csv_record(
    record: &csv::StringRecord,
    headers: &csv::StringRecord,
    field_map: &HashMap<String, String>
) -> Result<Contact> {
    let mut first_name = None;
    let mut last_name = None;
    let mut email = None;
    let mut phone = None;
    let mut organization = None;
    let mut title = None;

    for (idx, value) in record.iter().enumerate() {
        if let Some(header) = headers.get(idx) {
            if let Some(field) = field_map.get(header) {
                let val = if value.trim().is_empty() {
                    None
                } else {
                    Some(value.trim().to_string())
                };

                match field.as_str() {
                    "first_name" => first_name = val,
                    "last_name" => last_name = val,
                    "email" => email = val,
                    "phone" => phone = val,
                    "organization" => organization = val,
                    "title" => title = val,
                    _ => {}
                }
            }
        }
    }

    let first_name = first_name.ok_or_else(|| anyhow::anyhow!("Missing first_name"))?;

    // Use a placeholder user ID for CLI imports
    let placeholder_user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap();

    Ok(Contact {
        id: Uuid::new_v4(),
        first_name,
        last_name,
        email,
        phone,
        organization,
        title,
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
    })
}

pub async fn import_vcard(path: &str, _store: &LocalStore) -> Result<()> {
    let content = std::fs::read_to_string(path)?;
    println!("\n📇 vCard Import");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("File: {}", path);
    println!("Size: {} bytes", content.len());
    println!("\n⚠️  vCard parsing not yet implemented in alpha");
    println!("Beta release will support vCard 3.0/4.0 import with field mapping.");
    Ok(())
}

pub async fn import_sms(path: &str, store: &LocalStore) -> Result<()> {
    println!("\n📱 SMS Import (Android SMS Backup & Restore XML)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("File: {}", path);

    // Parse XML and extract contacts
    let contacts_map = parse_sms_xml(path)?;

    println!("\n📊 Import Summary:");
    println!("   Unique phone numbers: {}", contacts_map.len());

    // Show top 10 contacts by message count
    let mut contacts_vec: Vec<_> = contacts_map.iter().collect();
    contacts_vec.sort_by(|a, b| b.1.message_count.cmp(&a.1.message_count));

    println!("\n   Top contacts by message count:");
    for (phone, info) in contacts_vec.iter().take(10) {
        let name_display = if let Some(name) = &info.contact_name {
            if name != "(Unknown)" {
                name.clone()
            } else {
                "Unknown".to_string()
            }
        } else {
            "Unknown".to_string()
        };
        println!("     {} - {} messages (first: {}, last: {})",
            phone,
            info.message_count,
            info.first_message_date.format("%Y-%m-%d"),
            info.last_message_date.format("%Y-%m-%d")
        );
        if name_display != "Unknown" {
            println!("       Name from SMS backup: {}", name_display);
        }
    }

    print!("\n❓ Import {} contacts from SMS history? [y/N]: ", contacts_map.len());
    io::stdout().flush()?;
    let mut response = String::new();
    io::stdin().read_line(&mut response)?;

    if response.trim().to_lowercase() != "y" {
        println!("❌ Import cancelled");
        return Ok(());
    }

    println!("\n⏳ Importing contacts from SMS history...");
    let repo = ContactRepository::new(store.pool());
    let mut imported = 0;
    let mut skipped = 0;
    let mut failed = 0;

    for (phone, info) in contacts_map.iter() {
        // Extract name from contact_name field if available
        let (first_name, last_name) = if let Some(name) = &info.contact_name {
            if name != "(Unknown)" {
                parse_contact_name(name)
            } else {
                (phone.clone(), None)
            }
        } else {
            (phone.clone(), None)
        };

        let contact = Contact {
            id: Uuid::new_v4(),
            first_name,
            last_name,
            email: None,
            phone: Some(phone.clone()),
            organization: None,
            title: None,
            notes: Some(format!(
                "Imported from SMS backup - {} messages between {} and {}",
                info.message_count,
                info.first_message_date.format("%Y-%m-%d"),
                info.last_message_date.format("%Y-%m-%d")
            )),
            social_handles: vec![],
            tags: vec![],
            projects: vec![],
            groups: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(),
            version: 1,
            last_synced_at: None,
            metadata: serde_json::json!({"import_source": "sms_backup", "message_count": info.message_count}),
        };

        match repo.create(&contact).await {
            Ok(_) => {
                info!("Imported: {} ({})", contact.first_name, phone);
                imported += 1;
            }
            Err(e) => {
                if e.to_string().contains("UNIQUE constraint") {
                    skipped += 1;
                } else {
                    warn!("Failed to save contact {}: {}", phone, e);
                    failed += 1;
                }
            }
        }
    }

    println!("\n✅ SMS Import complete!");
    println!("   Imported: {}", imported);
    if skipped > 0 {
        println!("   Skipped (already exists): {}", skipped);
    }
    if failed > 0 {
        println!("   Failed: {}", failed);
    }

    Ok(())
}

#[derive(Debug)]
struct SmsContactInfo {
    contact_name: Option<String>,
    message_count: usize,
    first_message_date: DateTime<Utc>,
    last_message_date: DateTime<Utc>,
}

fn parse_sms_xml(path: &str) -> Result<HashMap<String, SmsContactInfo>> {
    let file = File::open(path)?;
    let file = BufReader::new(file);
    let mut reader = XmlReader::from_reader(file);
    reader.trim_text(true);

    let mut contacts: HashMap<String, SmsContactInfo> = HashMap::new();
    let mut buf = Vec::new();
    let mut total_messages = 0;

    println!("\n⏳ Parsing XML (this may take a moment for large files)...");

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(e)) if e.name().as_ref() == b"sms" => {
                total_messages += 1;
                if total_messages % 5000 == 0 {
                    print!("\r   Processed {} messages...", total_messages);
                    io::stdout().flush().ok();
                }

                let mut address = None;
                let mut date_ms = None;
                let mut contact_name = None;

                for attr in e.attributes().flatten() {
                    match attr.key.as_ref() {
                        b"address" => {
                            address = attr.unescape_value().ok().map(|v| v.to_string());
                        }
                        b"date" => {
                            if let Ok(ms_str) = attr.unescape_value() {
                                date_ms = ms_str.parse::<i64>().ok();
                            }
                        }
                        b"contact_name" => {
                            contact_name = attr.unescape_value().ok().map(|v| v.to_string());
                        }
                        _ => {}
                    }
                }

                if let (Some(addr), Some(ms)) = (address, date_ms) {
                    let phone = normalize_phone_number(&addr);
                    let date = DateTime::from_timestamp_millis(ms)
                        .unwrap_or_else(Utc::now);

                    contacts.entry(phone)
                        .and_modify(|info| {
                            info.message_count += 1;
                            if date < info.first_message_date {
                                info.first_message_date = date;
                            }
                            if date > info.last_message_date {
                                info.last_message_date = date;
                            }
                            // Update contact name if we don't have one yet
                            if info.contact_name.is_none() && contact_name.is_some() {
                                info.contact_name = contact_name.clone();
                            }
                        })
                        .or_insert(SmsContactInfo {
                            contact_name,
                            message_count: 1,
                            first_message_date: date,
                            last_message_date: date,
                        });
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                warn!("Error parsing XML at position {}: {}", reader.buffer_position(), e);
                break;
            }
            _ => {}
        }
        buf.clear();
    }

    println!("\r✅ Parsed {} total messages", total_messages);

    Ok(contacts)
}

fn normalize_phone_number(phone: &str) -> String {
    // Remove common formatting characters
    let phone = phone.replace(&[' ', '-', '(', ')', '.'][..], "");

    // If it starts with +, keep it
    if phone.starts_with('+') {
        return phone;
    }

    // If it's a short code (< 7 digits), return as-is
    if phone.chars().all(|c| c.is_numeric()) && phone.len() < 7 {
        return phone;
    }

    // For US numbers, add +1 if not present
    if phone.chars().all(|c| c.is_numeric()) && phone.len() == 10 {
        return format!("+1{}", phone);
    }

    phone
}

fn parse_contact_name(name: &str) -> (String, Option<String>) {
    let parts: Vec<&str> = name.split_whitespace().collect();
    match parts.len() {
        0 => (name.to_string(), None),
        1 => (parts[0].to_string(), None),
        _ => {
            let first = parts[0].to_string();
            let last = parts[1..].join(" ");
            (first, Some(last))
        }
    }
}