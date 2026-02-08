use anyhow::Result;
use chrono::{DateTime, Utc};
use core_domain::{
    Communication, CommunicationDirection, CommunicationHistoryStatus, CommunicationType, Contact,
};
use quick_xml::events::Event;
use quick_xml::Reader;
use serde_json::json;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Streaming SMS/Call XML parser for large files
/// Processes files chunk-by-chunk without loading entire file into memory
pub struct StreamingAndroidParser {
    batch_size: usize,
}

impl Default for StreamingAndroidParser {
    fn default() -> Self {
        Self { batch_size: 1000 }
    }
}

impl StreamingAndroidParser {
    pub fn new(batch_size: usize) -> Self {
        Self { batch_size }
    }

    /// Stream parse SMS XML file and call callback for each batch
    pub async fn parse_sms_stream<F, Fut>(
        &self,
        file_path: &Path,
        mut callback: F,
    ) -> Result<ParseStats>
    where
        F: FnMut(Vec<(Contact, Vec<Communication>)>) -> Fut,
        Fut: std::future::Future<Output = Result<()>>,
    {
        info!(
            "Starting streaming SMS parse from {:?} with batch size {}",
            file_path, self.batch_size
        );

        let file = File::open(file_path)?;
        let buf_reader = BufReader::new(file);
        let mut reader = Reader::from_reader(buf_reader);
        reader.trim_text(true);

        let mut stats = ParseStats::default();
        let mut current_batch: Vec<(Contact, Vec<Communication>)> = Vec::new();
        let mut contact_map: HashMap<String, usize> = HashMap::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e)) if e.name().as_ref() == b"sms" => {
                    stats.total_messages += 1;

                    if let Some((contact, communication)) =
                        self.parse_sms_element(&mut reader, e)?
                    {
                        let phone = contact.phone.clone().unwrap_or_default();

                        // Group by phone number to accumulate communications
                        if let Some(&idx) = contact_map.get(&phone) {
                            // Add communication to existing contact
                            current_batch[idx].1.push(communication);
                        } else {
                            // New contact
                            contact_map.insert(phone.clone(), current_batch.len());
                            current_batch.push((contact, vec![communication]));

                            // Process batch when it reaches batch_size
                            if current_batch.len() >= self.batch_size {
                                stats.contacts_created += current_batch.len();
                                callback(current_batch.clone()).await?;
                                current_batch.clear();
                                contact_map.clear();
                                debug!("Processed batch, total messages: {}", stats.total_messages);
                            }
                        }
                    } else {
                        stats.skipped += 1;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    warn!(
                        "XML parsing error at position {}: {:?}",
                        reader.buffer_position(),
                        e
                    );
                    stats.errors += 1;
                }
                _ => {}
            }
            buf.clear();
        }

        // Process remaining batch
        if !current_batch.is_empty() {
            stats.contacts_created += current_batch.len();
            callback(current_batch).await?;
        }

        info!(
            "Streaming SMS parse complete: {} messages, {} contacts, {} skipped, {} errors",
            stats.total_messages, stats.contacts_created, stats.skipped, stats.errors
        );

        Ok(stats)
    }

    /// Stream parse calls XML file
    pub async fn parse_calls_stream<F, Fut>(
        &self,
        file_path: &Path,
        mut callback: F,
    ) -> Result<ParseStats>
    where
        F: FnMut(Vec<(Contact, Vec<Communication>)>) -> Fut,
        Fut: std::future::Future<Output = Result<()>>,
    {
        info!(
            "Starting streaming calls parse from {:?} with batch size {}",
            file_path, self.batch_size
        );

        let file = File::open(file_path)?;
        let buf_reader = BufReader::new(file);
        let mut reader = Reader::from_reader(buf_reader);
        reader.trim_text(true);

        let mut stats = ParseStats::default();
        let mut current_batch: Vec<(Contact, Vec<Communication>)> = Vec::new();
        let mut contact_map: HashMap<String, usize> = HashMap::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e)) if e.name().as_ref() == b"call" => {
                    stats.total_messages += 1;

                    if let Some((contact, communication)) =
                        self.parse_call_element(&mut reader, e)?
                    {
                        let phone = contact.phone.clone().unwrap_or_default();

                        if let Some(&idx) = contact_map.get(&phone) {
                            // Add communication to existing contact
                            current_batch[idx].1.push(communication);
                        } else {
                            // New contact
                            contact_map.insert(phone.clone(), current_batch.len());
                            current_batch.push((contact, vec![communication]));

                            if current_batch.len() >= self.batch_size {
                                stats.contacts_created += current_batch.len();
                                callback(current_batch.clone()).await?;
                                current_batch.clear();
                                contact_map.clear();
                            }
                        }
                    } else {
                        stats.skipped += 1;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    warn!("XML parsing error: {:?}", e);
                    stats.errors += 1;
                }
                _ => {}
            }
            buf.clear();
        }

        if !current_batch.is_empty() {
            stats.contacts_created += current_batch.len();
            callback(current_batch).await?;
        }

        info!(
            "Streaming calls parse complete: {} calls, {} contacts",
            stats.total_messages, stats.contacts_created
        );

        Ok(stats)
    }

    fn parse_sms_element<R: std::io::BufRead>(
        &self,
        _reader: &mut Reader<R>,
        element: &quick_xml::events::BytesStart,
    ) -> Result<Option<(Contact, Communication)>> {
        let mut attrs: HashMap<String, String> = HashMap::new();

        for attr in element.attributes() {
            let attr = attr?;
            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
            let value = String::from_utf8_lossy(&attr.value).to_string();
            attrs.insert(key, value);
        }

        let address = match attrs.get("address") {
            Some(a) if !a.is_empty() && a != "null" => a.clone(),
            _ => return Ok(None),
        };

        let body = attrs.get("body").cloned().unwrap_or_default();
        let date_ms = attrs
            .get("date")
            .and_then(|d| d.parse::<i64>().ok())
            .unwrap_or(0);
        let sms_type = attrs.get("type").cloned().unwrap_or_default();
        let contact_name = attrs.get("contact_name").cloned();

        // Parse name from contact_name if available
        let (first_name, last_name) = if let Some(name) = contact_name.as_ref() {
            if name != "(Unknown)" && !name.is_empty() {
                let parts: Vec<&str> = name.splitn(2, ' ').collect();
                let first = parts.first().map(|s| s.to_string()).unwrap_or_default();
                let last = parts.get(1).map(|s| s.to_string());
                (first, last)
            } else {
                (String::new(), None)
            }
        } else {
            (String::new(), None)
        };

        let contact = Contact {
            id: Uuid::new_v4(),
            first_name,
            last_name,
            email: None,
            phone: Some(address.clone()),
            organization: None,
            title: None,
            notes: None,
            social_handles: vec![],
            tags: vec![],
            projects: vec![],
            groups: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: Uuid::nil(),
            version: 1,
            last_synced_at: None,
            metadata: json!({}),
        };

        // Create Communication record
        let direction = if sms_type == "2" {
            CommunicationDirection::Outbound
        } else {
            CommunicationDirection::Inbound
        };

        let timestamp = DateTime::from_timestamp_millis(date_ms).unwrap_or_else(Utc::now);

        let communication = Communication {
            id: Uuid::new_v4(),
            contact_id: contact.id,
            communication_type: CommunicationType::Sms,
            direction,
            timestamp,
            content: Some(body),
            duration_seconds: None,
            phone_number: Some(address.clone()),
            thread_id: Some(address.clone()), // Use phone number as thread ID for grouping
            status: CommunicationHistoryStatus::Completed,
            metadata: json!({
                "sms_type": sms_type,
                "contact_name": contact_name.unwrap_or_else(|| "(Unknown)".to_string()),
            }),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        Ok(Some((contact, communication)))
    }

    fn parse_call_element<R: std::io::BufRead>(
        &self,
        _reader: &mut Reader<R>,
        element: &quick_xml::events::BytesStart,
    ) -> Result<Option<(Contact, Communication)>> {
        let mut attrs: HashMap<String, String> = HashMap::new();

        for attr in element.attributes() {
            let attr = attr?;
            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
            let value = String::from_utf8_lossy(&attr.value).to_string();
            attrs.insert(key, value);
        }

        let number = match attrs.get("number") {
            Some(n) if !n.is_empty() => n.clone(),
            _ => return Ok(None),
        };

        let date_ms = attrs
            .get("date")
            .and_then(|d| d.parse::<i64>().ok())
            .unwrap_or(0);
        let duration = attrs
            .get("duration")
            .and_then(|d| d.parse::<i32>().ok())
            .unwrap_or(0);
        let call_type = attrs
            .get("type")
            .and_then(|t| t.parse::<i32>().ok())
            .unwrap_or(1);
        let contact_name = attrs.get("contact_name").cloned();

        let (first_name, last_name) = if let Some(name) = contact_name.as_ref() {
            if name != "(Unknown)" && !name.is_empty() {
                let parts: Vec<&str> = name.splitn(2, ' ').collect();
                let first = parts.first().map(|s| s.to_string()).unwrap_or_default();
                let last = parts.get(1).map(|s| s.to_string());
                (first, last)
            } else {
                (String::new(), None)
            }
        } else {
            (String::new(), None)
        };

        let contact = Contact {
            id: Uuid::new_v4(),
            first_name,
            last_name,
            email: None,
            phone: Some(number.clone()),
            organization: None,
            title: None,
            notes: None,
            social_handles: vec![],
            tags: vec![],
            projects: vec![],
            groups: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: Uuid::nil(),
            version: 1,
            last_synced_at: None,
            metadata: json!({}),
        };

        // Type: 1=incoming, 2=outgoing, 3=missed
        let direction = match call_type {
            2 => CommunicationDirection::Outbound,
            3 => CommunicationDirection::Missed,
            _ => CommunicationDirection::Inbound,
        };

        let timestamp = DateTime::from_timestamp_millis(date_ms).unwrap_or_else(Utc::now);

        let communication = Communication {
            id: Uuid::new_v4(),
            contact_id: contact.id,
            communication_type: CommunicationType::Call,
            direction,
            timestamp,
            content: Some(format!("Duration: {}s", duration)),
            duration_seconds: Some(duration),
            phone_number: Some(number.clone()),
            thread_id: Some(number.clone()), // Use phone number as thread ID
            status: CommunicationHistoryStatus::Completed,
            metadata: json!({
                "call_type": call_type,
                "contact_name": contact_name.unwrap_or_else(|| "(Unknown)".to_string()),
            }),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        Ok(Some((contact, communication)))
    }
}

#[derive(Debug, Default)]
pub struct ParseStats {
    pub total_messages: usize,
    pub contacts_created: usize,
    pub skipped: usize,
    pub errors: usize,
}
