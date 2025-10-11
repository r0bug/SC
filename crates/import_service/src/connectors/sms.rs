use crate::{
    config::ImportFormat,
    connector::{ConnectorMetadata, ImportConnector, ParseResult},
    ImportError,
};
use async_trait::async_trait;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;
use std::path::Path;

/// SMS Connector supporting Android SMS Backup & Restore XML and iOS CSV exports
pub struct SmsConnector;

impl SmsConnector {
    pub fn new() -> Self {
        Self
    }

    /// Parse Android SMS Backup & Restore XML format
    async fn parse_android_xml(&self, file_path: &Path) -> Result<ParseResult, ImportError> {
        let content = std::fs::read_to_string(file_path)?;
        let mut reader = Reader::from_str(&content);
        reader.trim_text(true);

        let mut rows = Vec::new();
        let mut warnings = Vec::new();
        let mut contacts_map: HashMap<String, HashMap<String, String>> = HashMap::new();

        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                    if e.name().as_ref() == b"sms" {
                        let mut phone = String::new();
                        let mut contact_name = String::new();
                        let mut message_body = String::new();
                        let mut message_type = String::new();
                        let mut date = String::new();

                        for attr in e.attributes() {
                            if let Ok(attr) = attr {
                                let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                                let value = String::from_utf8_lossy(&attr.value).to_string();

                                match key.as_str() {
                                    "address" => phone = value,
                                    "contact_name" => contact_name = value,
                                    "body" => message_body = value,
                                    "type" => message_type = value,
                                    "date" => date = value,
                                    _ => {}
                                }
                            }
                        }

                        // Create or update contact entry
                        let entry = contacts_map.entry(phone.clone()).or_insert_with(|| {
                            let mut map = HashMap::new();
                            map.insert("phone".to_string(), phone.clone());
                            if !contact_name.is_empty() && contact_name != "(Unknown)" {
                                map.insert("first_name".to_string(), contact_name.clone());
                            } else {
                                map.insert(
                                    "first_name".to_string(),
                                    format!("SMS Contact {}", phone),
                                );
                            }
                            map.insert("source".to_string(), "sms".to_string());
                            map.insert("notes".to_string(), String::new());
                            map
                        });

                        // Append message to notes
                        if !message_body.is_empty() {
                            let notes = entry.get("notes").cloned().unwrap_or_default();
                            let msg_type_label = if message_type == "1" {
                                "Received"
                            } else {
                                "Sent"
                            };
                            let timestamp = if !date.is_empty() {
                                // Convert milliseconds timestamp to readable date
                                if let Ok(ms) = date.parse::<i64>() {
                                    chrono::DateTime::from_timestamp(ms / 1000, 0)
                                        .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                                        .unwrap_or(date.clone())
                                } else {
                                    date.clone()
                                }
                            } else {
                                "Unknown".to_string()
                            };

                            let new_note = format!(
                                "{}\n[{}] {}: {}",
                                notes, timestamp, msg_type_label, message_body
                            );
                            entry.insert("notes".to_string(), new_note);
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    warnings.push(format!(
                        "XML parsing error at position {}: {}",
                        reader.buffer_position(),
                        e
                    ));
                }
                _ => {}
            }
            buf.clear();
        }

        // Convert map to rows
        rows.extend(contacts_map.into_values());

        let suggested_mappings = vec![
            ("phone".to_string(), "phone".to_string()),
            ("first_name".to_string(), "first_name".to_string()),
            ("notes".to_string(), "notes".to_string()),
        ];

        let mut metadata = HashMap::new();
        metadata.insert("format".to_string(), "android_sms_backup".to_string());
        metadata.insert("total_contacts".to_string(), rows.len().to_string());

        Ok(ParseResult {
            rows,
            suggested_mappings,
            metadata,
            warnings,
        })
    }

    /// Parse iOS SMS CSV export
    async fn parse_ios_csv(&self, file_path: &Path) -> Result<ParseResult, ImportError> {
        let mut reader = csv::Reader::from_path(file_path)?;
        let headers: Vec<String> = reader.headers()?.iter().map(|h| h.to_lowercase()).collect();

        let mut rows = Vec::new();
        let mut warnings = Vec::new();
        let mut contacts_map: HashMap<String, HashMap<String, String>> = HashMap::new();

        for (idx, result) in reader.records().enumerate() {
            match result {
                Ok(record) => {
                    let mut phone = String::new();
                    let mut contact_name = String::new();
                    let mut message = String::new();
                    let mut date = String::new();
                    let mut is_from_me = false;

                    for (i, field) in record.iter().enumerate() {
                        if let Some(header) = headers.get(i) {
                            match header.as_str() {
                                "phone number" | "phone_number" | "number" => {
                                    phone = field.to_string()
                                }
                                "contact" | "contact_name" | "name" => {
                                    contact_name = field.to_string()
                                }
                                "message" | "text" | "body" => message = field.to_string(),
                                "date" | "timestamp" | "time" => date = field.to_string(),
                                "is_from_me" | "sent" => {
                                    is_from_me = field == "1" || field.to_lowercase() == "true"
                                }
                                _ => {}
                            }
                        }
                    }

                    if phone.is_empty() {
                        warnings.push(format!("Row {} missing phone number, skipped", idx + 1));
                        continue;
                    }

                    // Create or update contact entry
                    let entry = contacts_map.entry(phone.clone()).or_insert_with(|| {
                        let mut map = HashMap::new();
                        map.insert("phone".to_string(), phone.clone());
                        if !contact_name.is_empty() {
                            map.insert("first_name".to_string(), contact_name.clone());
                        } else {
                            map.insert("first_name".to_string(), format!("SMS Contact {}", phone));
                        }
                        map.insert("source".to_string(), "sms".to_string());
                        map.insert("notes".to_string(), String::new());
                        map
                    });

                    // Append message to notes
                    if !message.is_empty() {
                        let notes = entry.get("notes").cloned().unwrap_or_default();
                        let msg_type = if is_from_me { "Sent" } else { "Received" };
                        let new_note = format!("{}\n[{}] {}: {}", notes, date, msg_type, message);
                        entry.insert("notes".to_string(), new_note);
                    }
                }
                Err(e) => {
                    warnings.push(format!("CSV parsing error at row {}: {}", idx + 1, e));
                }
            }
        }

        // Convert map to rows
        rows.extend(contacts_map.into_values());

        let suggested_mappings = vec![
            ("phone".to_string(), "phone".to_string()),
            ("first_name".to_string(), "first_name".to_string()),
            ("notes".to_string(), "notes".to_string()),
        ];

        let mut metadata = HashMap::new();
        metadata.insert("format".to_string(), "ios_sms_csv".to_string());
        metadata.insert("total_contacts".to_string(), rows.len().to_string());

        Ok(ParseResult {
            rows,
            suggested_mappings,
            metadata,
            warnings,
        })
    }

    /// Detect whether this is Android XML or iOS CSV
    fn detect_sms_format(file_path: &Path) -> Result<SmsFormat, ImportError> {
        let ext = file_path
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| ImportError::Other("No file extension".to_string()))?;

        match ext.to_lowercase().as_str() {
            "xml" => Ok(SmsFormat::AndroidXml),
            "csv" => Ok(SmsFormat::IosCsv),
            _ => Err(ImportError::Other(format!(
                "Unsupported SMS format: {}",
                ext
            ))),
        }
    }
}

#[derive(Debug, Clone)]
enum SmsFormat {
    AndroidXml,
    IosCsv,
}

#[async_trait]
impl ImportConnector for SmsConnector {
    fn metadata(&self) -> ConnectorMetadata {
        ConnectorMetadata {
            id: "sms".to_string(),
            name: "SMS Messages".to_string(),
            description:
                "Import contacts from SMS backup files (Android SMS Backup & Restore XML, iOS CSV)"
                    .to_string(),
            supported_extensions: vec!["xml".to_string(), "csv".to_string()],
            supported_mime_types: vec![
                "text/xml".to_string(),
                "application/xml".to_string(),
                "text/csv".to_string(),
            ],
            format: ImportFormat::Sms,
            required_auth_scopes: vec![],
            version: "1.0.0".to_string(),
        }
    }

    fn can_handle(&self, file_path: &Path) -> bool {
        // Check extension and try to detect content
        if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
            let ext_lower = ext.to_lowercase();
            if ext_lower == "xml" {
                // Check if it's an SMS backup XML by looking for <smses> tag
                if let Ok(content) = std::fs::read_to_string(file_path) {
                    return content.contains("<smses") || content.contains("<sms ");
                }
            } else if ext_lower == "csv" {
                // Check if it's an iOS SMS CSV by looking for common headers
                if let Ok(mut reader) = csv::Reader::from_path(file_path) {
                    if let Ok(headers) = reader.headers() {
                        let headers_str =
                            headers.iter().collect::<Vec<_>>().join(",").to_lowercase();
                        return headers_str.contains("phone")
                            && (headers_str.contains("message") || headers_str.contains("text"));
                    }
                }
            }
        }
        false
    }

    async fn parse(&self, file_path: &Path) -> Result<ParseResult, ImportError> {
        let format = Self::detect_sms_format(file_path)?;

        match format {
            SmsFormat::AndroidXml => self.parse_android_xml(file_path).await,
            SmsFormat::IosCsv => self.parse_ios_csv(file_path).await,
        }
    }

    async fn validate_file(&self, file_path: &Path) -> Result<(), ImportError> {
        if !file_path.exists() {
            return Err(ImportError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "File not found",
            )));
        }

        // Try to detect format
        Self::detect_sms_format(file_path)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_android_xml_parsing() {
        let xml_content = r#"<?xml version='1.0' encoding='UTF-8'?>
<smses count="2">
  <sms address="+1234567890" contact_name="John Doe" date="1609459200000" type="1" body="Hello there!" />
  <sms address="+1234567890" contact_name="John Doe" date="1609459260000" type="2" body="Hi back!" />
</smses>"#;

        let mut temp_file = NamedTempFile::with_suffix(".xml").unwrap();
        temp_file.write_all(xml_content.as_bytes()).unwrap();

        let connector = SmsConnector::new();
        let result = connector.parse(temp_file.path()).await.unwrap();

        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0].get("phone").unwrap(), "+1234567890");
        assert!(result.rows[0]
            .get("notes")
            .unwrap()
            .contains("Hello there!"));
    }

    #[tokio::test]
    async fn test_ios_csv_parsing() {
        let csv_content = "phone_number,contact_name,message,date,is_from_me\n\
+1234567890,Jane Smith,Test message,2024-01-01,false\n\
+1234567890,Jane Smith,Reply,2024-01-01,true";

        let mut temp_file = NamedTempFile::with_suffix(".csv").unwrap();
        temp_file.write_all(csv_content.as_bytes()).unwrap();

        let connector = SmsConnector::new();
        let result = connector.parse(temp_file.path()).await.unwrap();

        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0].get("phone").unwrap(), "+1234567890");
        assert_eq!(result.rows[0].get("first_name").unwrap(), "Jane Smith");
    }
}
