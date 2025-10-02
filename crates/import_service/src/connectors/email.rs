use crate::{
    connector::{ConnectorMetadata, ImportConnector, ParseResult},
    config::ImportFormat,
    ImportError,
};
use async_trait::async_trait;
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;

/// Email Connector supporting Gmail Takeout MBOX and Outlook CSV exports
pub struct EmailConnector;

impl EmailConnector {
    pub fn new() -> Self {
        Self
    }

    /// Parse Gmail Takeout MBOX format
    async fn parse_mbox(&self, file_path: &Path) -> Result<ParseResult, ImportError> {
        let content = std::fs::read_to_string(file_path)?;

        let mut rows = Vec::new();
        let mut warnings = Vec::new();
        let mut contacts_map: HashMap<String, HashMap<String, String>> = HashMap::new();

        // Regex patterns for email parsing
        let from_pattern = Regex::new(r"(?i)^From:\s*(.+)$").unwrap();
        let to_pattern = Regex::new(r"(?i)^To:\s*(.+)$").unwrap();
        let subject_pattern = Regex::new(r"(?i)^Subject:\s*(.+)$").unwrap();
        let date_pattern = Regex::new(r"(?i)^Date:\s*(.+)$").unwrap();
        let email_extract = Regex::new(r#"<?([a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,})>?"#).unwrap();
        let name_extract = Regex::new(r#"^["']?([^"'<]+?)["']?\s*<?[a-zA-Z0-9._%+-]+@"#).unwrap();

        let messages: Vec<&str> = content.split("\nFrom ").collect();

        for (idx, message) in messages.iter().enumerate() {
            let mut from_email = String::new();
            let mut from_name = String::new();
            let mut to_emails = Vec::new();
            let mut subject = String::new();
            let mut date = String::new();

            for line in message.lines().take(50) { // Only process header lines
                if let Some(caps) = from_pattern.captures(line) {
                    let from_str = caps.get(1).unwrap().as_str();
                    if let Some(email_cap) = email_extract.captures(from_str) {
                        from_email = email_cap.get(1).unwrap().as_str().to_string();
                    }
                    if let Some(name_cap) = name_extract.captures(from_str) {
                        from_name = name_cap.get(1).unwrap().as_str().trim().to_string();
                    }
                } else if let Some(caps) = to_pattern.captures(line) {
                    let to_str = caps.get(1).unwrap().as_str();
                    for email_cap in email_extract.captures_iter(to_str) {
                        to_emails.push(email_cap.get(1).unwrap().as_str().to_string());
                    }
                } else if let Some(caps) = subject_pattern.captures(line) {
                    subject = caps.get(1).unwrap().as_str().to_string();
                } else if let Some(caps) = date_pattern.captures(line) {
                    date = caps.get(1).unwrap().as_str().to_string();
                }
            }

            // Add sender as contact
            if !from_email.is_empty() {
                let entry = contacts_map.entry(from_email.clone()).or_insert_with(|| {
                    let mut map = HashMap::new();
                    map.insert("email".to_string(), from_email.clone());
                    if !from_name.is_empty() {
                        let parts: Vec<&str> = from_name.split_whitespace().collect();
                        if let Some(first) = parts.first() {
                            map.insert("first_name".to_string(), first.to_string());
                        }
                        if parts.len() > 1 {
                            if let Some(last) = parts.last() {
                                map.insert("last_name".to_string(), last.to_string());
                            }
                        }
                    } else {
                        map.insert("first_name".to_string(), from_email.split('@').next().unwrap_or("Unknown").to_string());
                    }
                    map.insert("source".to_string(), "email".to_string());
                    map.insert("notes".to_string(), String::new());
                    map
                });

                // Add email metadata to notes
                if !subject.is_empty() {
                    let notes = entry.get("notes").cloned().unwrap_or_default();
                    let new_note = format!(
                        "{}\n[{}] Email: {}",
                        notes,
                        if !date.is_empty() { &date } else { "Unknown date" },
                        subject
                    );
                    entry.insert("notes".to_string(), new_note);
                }
            }

            // Add recipients as contacts
            for to_email in to_emails {
                if !contacts_map.contains_key(&to_email) {
                    let mut map = HashMap::new();
                    map.insert("email".to_string(), to_email.clone());
                    map.insert("first_name".to_string(), to_email.split('@').next().unwrap_or("Unknown").to_string());
                    map.insert("source".to_string(), "email".to_string());
                    map.insert("notes".to_string(), "Email recipient".to_string());
                    contacts_map.insert(to_email, map);
                }
            }

            if idx % 1000 == 0 && idx > 0 {
                warnings.push(format!("Processed {} messages...", idx));
            }
        }

        rows.extend(contacts_map.into_values());

        let suggested_mappings = vec![
            ("email".to_string(), "email".to_string()),
            ("first_name".to_string(), "first_name".to_string()),
            ("last_name".to_string(), "last_name".to_string()),
            ("notes".to_string(), "notes".to_string()),
        ];

        let mut metadata = HashMap::new();
        metadata.insert("format".to_string(), "mbox".to_string());
        metadata.insert("total_contacts".to_string(), rows.len().to_string());
        metadata.insert("total_messages".to_string(), messages.len().to_string());

        Ok(ParseResult {
            rows,
            suggested_mappings,
            metadata,
            warnings,
        })
    }

    /// Parse Outlook CSV export
    async fn parse_outlook_csv(&self, file_path: &Path) -> Result<ParseResult, ImportError> {
        let mut reader = csv::Reader::from_path(file_path)?;
        let headers: Vec<String> = reader
            .headers()?
            .iter()
            .map(|h| h.to_string())
            .collect();

        let mut rows = Vec::new();
        let mut warnings = Vec::new();

        for (idx, result) in reader.records().enumerate() {
            match result {
                Ok(record) => {
                    let mut row = HashMap::new();

                    for (i, field) in record.iter().enumerate() {
                        if let Some(header) = headers.get(i) {
                            let normalized_key = match header.to_lowercase().as_str() {
                                "first name" | "given name" => "first_name",
                                "last name" | "surname" => "last_name",
                                "e-mail address" | "email address" | "email" => "email",
                                "business phone" | "home phone" | "mobile phone" | "phone" => "phone",
                                "company" | "company name" => "organization",
                                "job title" | "title" => "title",
                                "notes" | "comments" => "notes",
                                "business street" | "business address" => "business_address",
                                "business city" => "business_city",
                                "business state" => "business_state",
                                "business postal code" => "business_postal_code",
                                "business country" => "business_country",
                                _ => header.as_str(),
                            };

                            if !field.is_empty() {
                                row.insert(normalized_key.to_string(), field.to_string());
                            }
                        }
                    }

                    if !row.is_empty() {
                        // Ensure we have at least email or name
                        if row.contains_key("email") || row.contains_key("first_name") {
                            rows.push(row);
                        } else {
                            warnings.push(format!("Row {} missing email and name, skipped", idx + 1));
                        }
                    }
                }
                Err(e) => {
                    warnings.push(format!("CSV parsing error at row {}: {}", idx + 1, e));
                }
            }
        }

        // Auto-detect mappings based on what fields are present
        let mut suggested_mappings = vec![
            ("email".to_string(), "email".to_string()),
            ("first_name".to_string(), "first_name".to_string()),
            ("last_name".to_string(), "last_name".to_string()),
            ("phone".to_string(), "phone".to_string()),
            ("organization".to_string(), "organization".to_string()),
            ("title".to_string(), "title".to_string()),
            ("notes".to_string(), "notes".to_string()),
        ];

        let mut metadata = HashMap::new();
        metadata.insert("format".to_string(), "outlook_csv".to_string());
        metadata.insert("total_contacts".to_string(), rows.len().to_string());

        Ok(ParseResult {
            rows,
            suggested_mappings,
            metadata,
            warnings,
        })
    }

    /// Detect email format
    fn detect_email_format(file_path: &Path) -> Result<EmailFormat, ImportError> {
        let ext = file_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase());

        match ext.as_deref() {
            Some("mbox") => Ok(EmailFormat::Mbox),
            Some("csv") => {
                // Check if it's Outlook CSV by looking at headers
                if let Ok(mut reader) = csv::Reader::from_path(file_path) {
                    if let Ok(headers) = reader.headers() {
                        let headers_str = headers.iter().collect::<Vec<_>>().join(",").to_lowercase();
                        if headers_str.contains("e-mail") || headers_str.contains("business phone") {
                            return Ok(EmailFormat::OutlookCsv);
                        }
                    }
                }
                Err(ImportError::Other("Could not detect Outlook CSV format".to_string()))
            }
            Some("pst") => Ok(EmailFormat::Pst),
            _ => Err(ImportError::Other("Unsupported email format".to_string())),
        }
    }
}

#[derive(Debug, Clone)]
enum EmailFormat {
    Mbox,
    OutlookCsv,
    Pst,
}

#[async_trait]
impl ImportConnector for EmailConnector {
    fn metadata(&self) -> ConnectorMetadata {
        ConnectorMetadata {
            id: "email".to_string(),
            name: "Email Contacts".to_string(),
            description: "Import contacts from email exports (Gmail Takeout MBOX, Outlook CSV)".to_string(),
            supported_extensions: vec!["mbox".to_string(), "csv".to_string()],
            supported_mime_types: vec![
                "application/mbox".to_string(),
                "text/csv".to_string(),
                "application/vnd.ms-outlook".to_string(),
            ],
            format: ImportFormat::Email,
            required_auth_scopes: vec![],
            version: "1.0.0".to_string(),
        }
    }

    fn can_handle(&self, file_path: &Path) -> bool {
        if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
            let ext_lower = ext.to_lowercase();
            if ext_lower == "mbox" {
                return true;
            } else if ext_lower == "csv" {
                // Check for Outlook-specific headers
                if let Ok(mut reader) = csv::Reader::from_path(file_path) {
                    if let Ok(headers) = reader.headers() {
                        let headers_str = headers.iter().collect::<Vec<_>>().join(",").to_lowercase();
                        return headers_str.contains("e-mail") || headers_str.contains("business phone");
                    }
                }
            }
        }
        false
    }

    async fn parse(&self, file_path: &Path) -> Result<ParseResult, ImportError> {
        let format = Self::detect_email_format(file_path)?;

        match format {
            EmailFormat::Mbox => self.parse_mbox(file_path).await,
            EmailFormat::OutlookCsv => self.parse_outlook_csv(file_path).await,
            EmailFormat::Pst => {
                Err(ImportError::Other(
                    "PST format not yet supported. Please export to CSV or MSG format first. \
                     See documentation for conversion instructions.".to_string()
                ))
            }
        }
    }

    async fn validate_file(&self, file_path: &Path) -> Result<(), ImportError> {
        if !file_path.exists() {
            return Err(ImportError::IoError(
                std::io::Error::new(std::io::ErrorKind::NotFound, "File not found")
            ));
        }

        Self::detect_email_format(file_path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_mbox_parsing() {
        let mbox_content = r#"From test@example.com Mon Jan 01 00:00:00 2024
From: "John Doe" <john@example.com>
To: jane@example.com
Subject: Test Email
Date: Mon, 1 Jan 2024 00:00:00 +0000

Test message body
"#;

        let mut temp_file = NamedTempFile::with_suffix(".mbox").unwrap();
        write!(temp_file, "{}", mbox_content).unwrap();

        let connector = EmailConnector::new();
        let result = connector.parse(temp_file.path()).await.unwrap();

        assert!(result.rows.len() >= 1);
        assert!(result.rows.iter().any(|r| r.get("email").unwrap() == "john@example.com"));
    }

    #[tokio::test]
    async fn test_outlook_csv_parsing() {
        let csv_content = "First Name,Last Name,E-mail Address,Business Phone,Company\n\
John,Doe,john@example.com,555-1234,Acme Corp";

        let mut temp_file = NamedTempFile::with_suffix(".csv").unwrap();
        write!(temp_file, "{}", csv_content).unwrap();

        let connector = EmailConnector::new();
        let result = connector.parse(temp_file.path()).await.unwrap();

        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0].get("email").unwrap(), "john@example.com");
        assert_eq!(result.rows[0].get("organization").unwrap(), "Acme Corp");
    }
}
