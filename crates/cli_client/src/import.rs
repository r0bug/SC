use anyhow::Result;
use chrono::{DateTime, Utc};
use core_domain::*;
use csv::Reader;
use local_store::{ContactRepository, LocalStore};
use quick_xml::events::Event;
use quick_xml::Reader as XmlReader;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader, Write};
use tracing::{info, warn};
use uuid::Uuid;

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
            Err(e) => errors.push(format!("Row {}: {}", idx + 2, e)),
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
            Ok(contact) => match repo.create(&contact).await {
                Ok(_) => {
                    info!(
                        "Imported: {} {}",
                        contact.first_name,
                        contact.last_name.as_deref().unwrap_or("")
                    );
                    imported += 1;
                }
                Err(e) => {
                    warn!("Failed to save contact: {}", e);
                    failed += 1;
                }
            },
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
    field_map: &HashMap<String, String>,
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

    // Parse XML and extract individual SMS messages
    let sms_data = parse_sms_messages(path)?;

    println!("\n📊 Import Summary:");
    println!("   Total SMS messages: {}", sms_data.total_messages);
    println!("   Unique phone numbers: {}", sms_data.messages_by_phone.len());

    // Show top 10 contacts by message count
    let mut phone_stats: Vec<_> = sms_data.messages_by_phone.iter()
        .map(|(phone, messages)| (phone, messages.len()))
        .collect();
    phone_stats.sort_by(|a, b| b.1.cmp(&a.1));

    println!("\n   Top contacts by message count:");
    for (phone, count) in phone_stats.iter().take(10) {
        println!("     {} - {} messages", phone, count);
    }

    print!(
        "\n❓ Import {} phone numbers and {} SMS messages? [y/N]: ",
        sms_data.messages_by_phone.len(),
        sms_data.total_messages
    );
    io::stdout().flush()?;
    let mut response = String::new();
    io::stdin().read_line(&mut response)?;

    if response.trim().to_lowercase() != "y" {
        println!("❌ Import cancelled");
        return Ok(());
    }

    println!("\n⏳ Importing contacts and SMS history...");
    let contact_repo = ContactRepository::new(store.pool());
    let sms_repo = local_store::SmsHistoryRepository::new(store.pool());

    let mut contacts_created = 0;
    let mut contacts_found = 0;
    let mut messages_imported = 0;
    let mut failed = 0;

    let default_user = Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap();

    for (phone, messages) in sms_data.messages_by_phone.iter() {
        // Look up existing contact by phone number
        let contact = match contact_repo.get_by_phone(phone).await? {
            Some(existing) => {
                info!("Found existing contact for {}: {} {}", phone, existing.first_name, existing.last_name.as_deref().unwrap_or(""));
                contacts_found += 1;
                existing
            }
            None => {
                // Create new contact
                let (first_name, last_name) = if let Some(first_msg) = messages.first() {
                    if let Some(name) = &first_msg.contact_name {
                        if name != "(Unknown)" {
                            parse_contact_name(name)
                        } else {
                            (phone.clone(), None)
                        }
                    } else {
                        (phone.clone(), None)
                    }
                } else {
                    (phone.clone(), None)
                };

                let contact = Contact {
                    id: Uuid::new_v4(),
                    first_name: first_name.clone(),
                    last_name,
                    email: None,
                    phone: Some(phone.clone()),
                    organization: None,
                    title: None,
                    notes: Some(format!(
                        "Imported from SMS backup - {} messages",
                        messages.len()
                    )),
                    social_handles: vec![],
                    tags: vec![],
                    projects: vec![],
                    groups: vec![],
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    created_by: default_user,
                    version: 1,
                    last_synced_at: None,
                    metadata: serde_json::json!({"import_source": "sms_backup", "message_count": messages.len()}),
                };

                match contact_repo.create(&contact).await {
                    Ok(_) => {
                        info!("Created contact: {} ({})", first_name, phone);
                        contacts_created += 1;
                        contact
                    }
                    Err(e) => {
                        warn!("Failed to create contact for {}: {}", phone, e);
                        failed += 1;
                        continue;
                    }
                }
            }
        };

        // Convert parsed SMS messages to SmsMessage structs and batch import
        let sms_messages: Vec<local_store::SmsMessage> = messages
            .iter()
            .map(|msg| local_store::SmsMessage {
                id: Uuid::new_v4(),
                contact_id: Some(contact.id),
                phone_number: phone.clone(),
                contact_name: msg.contact_name.clone(),
                message_date: msg.date_ms,
                message_type: msg.message_type,
                subject: None,
                body: msg.body.clone(),
                readable_date: msg.readable_date.clone(),
                thread_id: msg.thread_id,
                read_status: if msg.read == 1 { 1 } else { 0 },
                subscription_id: msg.subscription_id.clone(),
                imported_at: Utc::now(),
                imported_by: "cli_import".to_string(),
                source_file: Some(path.to_string()),
            })
            .collect();

        // Batch insert SMS messages
        match sms_repo.batch_create(&sms_messages).await {
            Ok(count) => {
                messages_imported += count;
            }
            Err(e) => {
                warn!("Failed to import SMS messages for {}: {}", phone, e);
                failed += 1;
            }
        }

        if (contacts_created + contacts_found) % 100 == 0 {
            print!("\r   Processed {} phone numbers...", contacts_created + contacts_found);
            io::stdout().flush().ok();
        }
    }

    println!("\n✅ SMS Import complete!");
    println!("   Contacts created: {}", contacts_created);
    println!("   Existing contacts found: {}", contacts_found);
    println!("   SMS messages imported: {}", messages_imported);
    if failed > 0 {
        println!("   Failed: {}", failed);
    }

    Ok(())
}

#[derive(Debug, Clone)]
struct ParsedSmsMessage {
    contact_name: Option<String>,
    body: String,
    date_ms: i64,
    readable_date: String,
    message_type: i32,  // 1=received, 2=sent, 3=draft, etc.
    thread_id: Option<i32>,
    read: i32,
    subscription_id: Option<String>,
}

#[derive(Debug)]
struct SmsImportData {
    messages_by_phone: HashMap<String, Vec<ParsedSmsMessage>>,
    total_messages: usize,
}

fn parse_sms_messages(path: &str) -> Result<SmsImportData> {
    let file = File::open(path)?;
    let file = BufReader::new(file);
    let mut reader = XmlReader::from_reader(file);
    reader.trim_text(true);

    let mut messages_by_phone: HashMap<String, Vec<ParsedSmsMessage>> = HashMap::new();
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
                let mut body = String::new();
                let mut msg_type = 1; // default to received
                let mut thread_id = None;
                let mut read = 1;
                let mut subscription_id = None;

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
                        b"body" => {
                            body = attr.unescape_value().ok().map(|v| v.to_string()).unwrap_or_default();
                        }
                        b"type" => {
                            if let Ok(type_str) = attr.unescape_value() {
                                msg_type = type_str.parse::<i32>().unwrap_or(1);
                            }
                        }
                        b"thread_id" => {
                            if let Ok(thread_str) = attr.unescape_value() {
                                thread_id = thread_str.parse::<i32>().ok();
                            }
                        }
                        b"read" => {
                            if let Ok(read_str) = attr.unescape_value() {
                                read = read_str.parse::<i32>().unwrap_or(1);
                            }
                        }
                        b"sub_id" | b"subscription_id" => {
                            subscription_id = attr.unescape_value().ok().map(|v| v.to_string());
                        }
                        _ => {}
                    }
                }

                if let (Some(addr), Some(ms)) = (address, date_ms) {
                    let phone = normalize_phone_number(&addr);

                    // Convert timestamp to readable date
                    let readable_date = DateTime::from_timestamp_millis(ms)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_else(|| "Unknown".to_string());

                    let message = ParsedSmsMessage {
                        contact_name,
                        body,
                        date_ms: ms,
                        readable_date,
                        message_type: msg_type,
                        thread_id,
                        read,
                        subscription_id,
                    };

                    messages_by_phone
                        .entry(phone)
                        .or_insert_with(Vec::new)
                        .push(message);
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                warn!(
                    "Error parsing XML at position {}: {}",
                    reader.buffer_position(),
                    e
                );
                break;
            }
            _ => {}
        }
        buf.clear();
    }

    println!("\r✅ Parsed {} total messages", total_messages);

    Ok(SmsImportData {
        messages_by_phone,
        total_messages,
    })
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
