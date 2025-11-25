use anyhow::Result;
use chrono::Utc;
use core_domain::{Contact, system_user_id};
use csv::ReaderBuilder;
use serde_json::json;
use std::fs::File;
use std::path::Path;
use tracing::{info, warn};
use uuid::Uuid;

/// Google Contacts CSV importer
pub struct GoogleContactsImporter;

impl GoogleContactsImporter {
    pub fn parse_csv(file_path: &Path, user_id: Uuid) -> Result<Vec<Contact>> {
        info!("Parsing Google Contacts CSV from {:?}", file_path);

        let file = File::open(file_path)?;
        let mut reader = ReaderBuilder::new().from_reader(file);

        let mut contacts = Vec::new();
        let mut skipped = 0;

        for result in reader.deserialize::<GoogleContactRow>() {
            match result {
                Ok(row) => {
                    if let Some(contact) = Self::row_to_contact(row, user_id) {
                        contacts.push(contact);
                    } else {
                        skipped += 1;
                    }
                }
                Err(e) => {
                    warn!("Error parsing CSV row: {}", e);
                    skipped += 1;
                }
            }
        }

        info!(
            "Parsed {} contacts from Google Contacts CSV ({} skipped)",
            contacts.len(),
            skipped
        );

        Ok(contacts)
    }

    fn row_to_contact(row: GoogleContactRow, user_id: Uuid) -> Option<Contact> {
        // Skip if no meaningful data
        let has_name = !row.first_name.is_empty() || !row.last_name.is_empty();
        let has_email = !row.email_1_value.is_empty();
        let has_phone = !row.phone_1_value.is_empty();
        let has_org = row.organization_name.is_some();

        if !has_name && !has_email && !has_phone && !has_org {
            return None;
        }

        // Clean phone number
        let phone = if !row.phone_1_value.is_empty() {
            Some(Self::clean_phone(&row.phone_1_value))
        } else if !row.phone_2_value.is_empty() {
            Some(Self::clean_phone(&row.phone_2_value))
        } else {
            None
        };

        // Clean email
        let email = if !row.email_1_value.is_empty() {
            Some(row.email_1_value.trim().to_lowercase())
        } else if !row.email_2_value.is_empty() {
            Some(row.email_2_value.trim().to_lowercase())
        } else {
            None
        };

        let metadata = json!({
            "google_notes": if !row.notes.is_empty() { row.notes.as_str() } else { "" },
            "google_labels": if !row.labels.is_empty() { row.labels.as_str() } else { "" }
        });

        Some(Contact {
            id: Uuid::new_v4(),
            first_name: row.first_name.trim().to_string(),
            last_name: if row.last_name.is_empty() {
                None
            } else {
                Some(row.last_name.trim().to_string())
            },
            email,
            phone,
            organization: row.organization_name,
            title: row.organization_title,
            notes: if row.notes.is_empty() {
                None
            } else {
                Some(row.notes)
            },
            social_handles: vec![],
            tags: vec![],
            projects: vec![],
            groups: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: user_id,
            version: 1,
            last_synced_at: None,
            metadata,
        })
    }

    fn clean_phone(phone: &str) -> String {
        // Remove all non-digit characters except + at start
        let mut cleaned = String::new();
        let mut chars = phone.chars().peekable();

        // Keep leading +
        if chars.peek() == Some(&'+') {
            cleaned.push('+');
            chars.next();
        }

        // Keep only digits
        for c in chars {
            if c.is_ascii_digit() {
                cleaned.push(c);
            }
        }

        cleaned
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
struct GoogleContactRow {
    #[serde(rename = "First Name", default)]
    first_name: String,

    #[serde(rename = "Last Name", default)]
    last_name: String,

    #[serde(rename = "Organization Name", default)]
    organization_name: Option<String>,

    #[serde(rename = "Organization Title", default)]
    organization_title: Option<String>,

    #[serde(rename = "Notes", default)]
    notes: String,

    #[serde(rename = "Labels", default)]
    labels: String,

    #[serde(rename = "E-mail 1 - Value", default)]
    email_1_value: String,

    #[serde(rename = "E-mail 2 - Value", default)]
    email_2_value: String,

    #[serde(rename = "Phone 1 - Value", default)]
    phone_1_value: String,

    #[serde(rename = "Phone 2 - Value", default)]
    phone_2_value: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_phone() {
        assert_eq!(GoogleContactsImporter::clean_phone("+1-509-952-7982"), "+15099527982");
        assert_eq!(GoogleContactsImporter::clean_phone("509-952-7982"), "5099527982");
        assert_eq!(GoogleContactsImporter::clean_phone("+1 (509) 952-7982"), "+15099527982");
    }
}
