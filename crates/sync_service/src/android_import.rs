use anyhow::{anyhow, Result};
use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use std::io::BufRead;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallRecord {
    pub id: String,
    pub phone_number: String,
    pub contact_name: Option<String>,
    pub call_date: i64,
    pub duration: i64,
    pub call_type: i32,
    pub readable_date: String,
    pub subscription_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsRecord {
    pub id: String,
    pub phone_number: String,
    pub contact_name: Option<String>,
    pub message_date: i64,
    pub message_type: i32,
    pub subject: Option<String>,
    pub body: String,
    pub readable_date: String,
    pub thread_id: Option<i64>,
    pub read_status: i32,
    pub subscription_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MmsRecord {
    pub id: String,
    pub phone_number: String,
    pub contact_name: Option<String>,
    pub message_date: i64,
    pub message_type: i32,
    pub subject: Option<String>,
    pub readable_date: String,
    pub thread_id: Option<i64>,
    pub read_status: i32,
    pub subscription_id: Option<String>,
    pub parts: Vec<MmsPart>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MmsPart {
    pub sequence: i32,
    pub content_type: String,
    pub name: Option<String>,
    pub text: Option<String>,
}

/// Parse Android SMS Backup & Restore XML file for calls
pub fn parse_calls_xml<R: BufRead>(reader: R) -> Result<Vec<CallRecord>> {
    let mut xml_reader = Reader::from_reader(reader);
    xml_reader.trim_text(true);

    let mut calls = Vec::new();
    let mut buf = Vec::new();

    loop {
        match xml_reader.read_event_into(&mut buf) {
            Ok(Event::Empty(e)) if e.name().as_ref() == b"call" => {
                let mut call = CallRecord {
                    id: Uuid::new_v4().to_string(),
                    phone_number: String::new(),
                    contact_name: None,
                    call_date: 0,
                    duration: 0,
                    call_type: 1,
                    readable_date: String::new(),
                    subscription_id: None,
                };

                for attr in e.attributes() {
                    let attr = attr?;
                    let key = std::str::from_utf8(attr.key.as_ref())?;
                    let value = attr.unescape_value()?;

                    match key {
                        "number" => call.phone_number = value.to_string(),
                        "contact_name" => call.contact_name = Some(value.to_string()),
                        "date" => call.call_date = value.parse().unwrap_or(0),
                        "duration" => call.duration = value.parse().unwrap_or(0),
                        "type" => call.call_type = value.parse().unwrap_or(1),
                        "readable_date" => call.readable_date = value.to_string(),
                        "subscription_id" => call.subscription_id = Some(value.to_string()),
                        _ => {}
                    }
                }

                if !call.phone_number.is_empty() {
                    calls.push(call);
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow!("Error parsing XML: {:?}", e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(calls)
}

/// Parse Android SMS Backup & Restore XML file for SMS
pub fn parse_sms_xml<R: BufRead>(reader: R) -> Result<Vec<SmsRecord>> {
    let mut xml_reader = Reader::from_reader(reader);
    xml_reader.trim_text(true);

    let mut messages = Vec::new();
    let mut buf = Vec::new();

    loop {
        match xml_reader.read_event_into(&mut buf) {
            Ok(Event::Empty(e)) if e.name().as_ref() == b"sms" => {
                let mut sms = SmsRecord {
                    id: Uuid::new_v4().to_string(),
                    phone_number: String::new(),
                    contact_name: None,
                    message_date: 0,
                    message_type: 1,
                    subject: None,
                    body: String::new(),
                    readable_date: String::new(),
                    thread_id: None,
                    read_status: 1,
                    subscription_id: None,
                };

                for attr in e.attributes() {
                    let attr = attr?;
                    let key = std::str::from_utf8(attr.key.as_ref())?;
                    let value = attr.unescape_value()?;

                    match key {
                        "address" => sms.phone_number = value.to_string(),
                        "contact_name" => sms.contact_name = Some(value.to_string()),
                        "date" => sms.message_date = value.parse().unwrap_or(0),
                        "type" => sms.message_type = value.parse().unwrap_or(1),
                        "subject" => sms.subject = Some(value.to_string()),
                        "body" => sms.body = value.to_string(),
                        "readable_date" => sms.readable_date = value.to_string(),
                        "thread_id" => sms.thread_id = value.parse().ok(),
                        "read" => sms.read_status = value.parse().unwrap_or(1),
                        "subscription_id" => sms.subscription_id = Some(value.to_string()),
                        _ => {}
                    }
                }

                if !sms.phone_number.is_empty() {
                    messages.push(sms);
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow!("Error parsing XML: {:?}", e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(messages)
}

/// Parse Android SMS Backup & Restore XML file for MMS
pub fn parse_mms_xml<R: BufRead>(reader: R) -> Result<Vec<MmsRecord>> {
    let mut xml_reader = Reader::from_reader(reader);
    xml_reader.trim_text(true);

    let mut messages = Vec::new();
    let mut buf = Vec::new();
    let mut current_mms: Option<MmsRecord> = None;

    loop {
        match xml_reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) if e.name().as_ref() == b"mms" => {
                let mut mms = MmsRecord {
                    id: Uuid::new_v4().to_string(),
                    phone_number: String::new(),
                    contact_name: None,
                    message_date: 0,
                    message_type: 1,
                    subject: None,
                    readable_date: String::new(),
                    thread_id: None,
                    read_status: 1,
                    subscription_id: None,
                    parts: Vec::new(),
                };

                for attr in e.attributes() {
                    let attr = attr?;
                    let key = std::str::from_utf8(attr.key.as_ref())?;
                    let value = attr.unescape_value()?;

                    match key {
                        "address" => mms.phone_number = value.to_string(),
                        "contact_name" => mms.contact_name = Some(value.to_string()),
                        "date" => mms.message_date = value.parse().unwrap_or(0),
                        "msg_box" => mms.message_type = value.parse().unwrap_or(1),
                        "subject" => mms.subject = Some(value.to_string()),
                        "readable_date" => mms.readable_date = value.to_string(),
                        "thread_id" => mms.thread_id = value.parse().ok(),
                        "read" => mms.read_status = value.parse().unwrap_or(1),
                        "subscription_id" => mms.subscription_id = Some(value.to_string()),
                        _ => {}
                    }
                }

                current_mms = Some(mms);
            }
            Ok(Event::Empty(e)) if e.name().as_ref() == b"part" => {
                if let Some(ref mut mms) = current_mms {
                    let mut part = MmsPart {
                        sequence: 0,
                        content_type: String::new(),
                        name: None,
                        text: None,
                    };

                    for attr in e.attributes() {
                        let attr = attr?;
                        let key = std::str::from_utf8(attr.key.as_ref())?;
                        let value = attr.unescape_value()?;

                        match key {
                            "seq" => part.sequence = value.parse().unwrap_or(0),
                            "ct" => part.content_type = value.to_string(),
                            "name" => part.name = Some(value.to_string()),
                            "text" => part.text = Some(value.to_string()),
                            _ => {}
                        }
                    }

                    mms.parts.push(part);
                }
            }
            Ok(Event::End(e)) if e.name().as_ref() == b"mms" => {
                if let Some(mms) = current_mms.take() {
                    if !mms.phone_number.is_empty() {
                        messages.push(mms);
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow!("Error parsing XML: {:?}", e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(messages)
}

/// Insert call records into database
pub async fn insert_calls(
    pool: &Pool<Sqlite>,
    calls: Vec<CallRecord>,
    user_id: &str,
    source_file: &str,
) -> Result<(usize, usize)> {
    let imported_at = chrono::Utc::now().to_rfc3339();
    let mut inserted = 0;
    let mut skipped = 0;

    for call in calls {
        // Try to match phone number to existing contact
        let contact_id: Option<String> = sqlx::query_scalar(
            "SELECT id FROM contacts WHERE phone = ? LIMIT 1"
        )
        .bind(&call.phone_number)
        .fetch_optional(pool)
        .await?;

        // Insert call record
        let result = sqlx::query(
            r#"
            INSERT INTO call_history
            (id, contact_id, phone_number, contact_name, call_date, duration,
             call_type, readable_date, subscription_id, imported_at, imported_by, source_file)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&call.id)
        .bind(contact_id)
        .bind(&call.phone_number)
        .bind(&call.contact_name)
        .bind(call.call_date)
        .bind(call.duration)
        .bind(call.call_type)
        .bind(&call.readable_date)
        .bind(&call.subscription_id)
        .bind(&imported_at)
        .bind(user_id)
        .bind(source_file)
        .execute(pool)
        .await;

        match result {
            Ok(_) => inserted += 1,
            Err(_) => skipped += 1,
        }
    }

    Ok((inserted, skipped))
}

/// Insert SMS records into database
pub async fn insert_sms(
    pool: &Pool<Sqlite>,
    messages: Vec<SmsRecord>,
    user_id: &str,
    source_file: &str,
) -> Result<(usize, usize)> {
    let imported_at = chrono::Utc::now().to_rfc3339();
    let mut inserted = 0;
    let mut skipped = 0;

    for sms in messages {
        // Try to match phone number to existing contact
        let contact_id: Option<String> = sqlx::query_scalar(
            "SELECT id FROM contacts WHERE phone = ? LIMIT 1"
        )
        .bind(&sms.phone_number)
        .fetch_optional(pool)
        .await?;

        // Insert SMS record
        let result = sqlx::query(
            r#"
            INSERT INTO sms_history
            (id, contact_id, phone_number, contact_name, message_date, message_type,
             subject, body, readable_date, thread_id, read_status, subscription_id,
             imported_at, imported_by, source_file)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&sms.id)
        .bind(contact_id)
        .bind(&sms.phone_number)
        .bind(&sms.contact_name)
        .bind(sms.message_date)
        .bind(sms.message_type)
        .bind(&sms.subject)
        .bind(&sms.body)
        .bind(&sms.readable_date)
        .bind(sms.thread_id)
        .bind(sms.read_status)
        .bind(&sms.subscription_id)
        .bind(&imported_at)
        .bind(user_id)
        .bind(source_file)
        .execute(pool)
        .await;

        match result {
            Ok(_) => inserted += 1,
            Err(_) => skipped += 1,
        }
    }

    Ok((inserted, skipped))
}

/// Insert MMS records into database
pub async fn insert_mms(
    pool: &Pool<Sqlite>,
    messages: Vec<MmsRecord>,
    user_id: &str,
    source_file: &str,
) -> Result<(usize, usize)> {
    let imported_at = chrono::Utc::now().to_rfc3339();
    let mut inserted = 0;
    let mut skipped = 0;

    for mms in messages {
        // Try to match phone number to existing contact
        let contact_id: Option<String> = sqlx::query_scalar(
            "SELECT id FROM contacts WHERE phone = ? LIMIT 1"
        )
        .bind(&mms.phone_number)
        .fetch_optional(pool)
        .await?;

        // Combine MMS parts into body text
        let body_parts: Vec<String> = mms.parts.iter()
            .filter_map(|p| p.text.clone())
            .collect();
        let body = body_parts.join("\n");

        // Insert MMS record as SMS
        let result = sqlx::query(
            r#"
            INSERT INTO sms_history
            (id, contact_id, phone_number, contact_name, message_date, message_type,
             subject, body, readable_date, thread_id, read_status, subscription_id,
             imported_at, imported_by, source_file)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&mms.id)
        .bind(contact_id)
        .bind(&mms.phone_number)
        .bind(&mms.contact_name)
        .bind(mms.message_date)
        .bind(mms.message_type)
        .bind(&mms.subject)
        .bind(&body)
        .bind(&mms.readable_date)
        .bind(mms.thread_id)
        .bind(mms.read_status)
        .bind(&mms.subscription_id)
        .bind(&imported_at)
        .bind(user_id)
        .bind(source_file)
        .execute(pool)
        .await;

        if result.is_ok() {
            // Insert MMS parts
            for (idx, part) in mms.parts.iter().enumerate() {
                let part_id = Uuid::new_v4().to_string();
                let _ = sqlx::query(
                    r#"
                    INSERT INTO mms_parts
                    (id, sms_id, sequence, content_type, name, text)
                    VALUES (?, ?, ?, ?, ?, ?)
                    "#
                )
                .bind(&part_id)
                .bind(&mms.id)
                .bind(idx as i32)
                .bind(&part.content_type)
                .bind(&part.name)
                .bind(&part.text)
                .execute(pool)
                .await;
            }
            inserted += 1;
        } else {
            skipped += 1;
        }
    }

    Ok((inserted, skipped))
}
