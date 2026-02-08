use crate::{
    config::ImportFormat,
    connector::{ConnectorMetadata, ImportConnector, ParseResult},
    ImportError,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;

/// Google Contacts CSV Connector
pub struct GoogleContactsConnector;

impl GoogleContactsConnector {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ImportConnector for GoogleContactsConnector {
    fn metadata(&self) -> ConnectorMetadata {
        ConnectorMetadata {
            id: "google_contacts".to_string(),
            name: "Google Contacts".to_string(),
            description: "Import contacts from Google Contacts CSV export".to_string(),
            supported_extensions: vec!["csv".to_string()],
            supported_mime_types: vec!["text/csv".to_string()],
            format: ImportFormat::Csv,
            required_auth_scopes: vec![],
            version: "1.0.0".to_string(),
        }
    }

    fn can_handle(&self, file_path: &Path) -> bool {
        if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
            if ext.to_lowercase() == "csv" {
                // Check for Google Contacts specific headers
                if let Ok(mut reader) = csv::Reader::from_path(file_path) {
                    if let Ok(headers) = reader.headers() {
                        let headers_str = headers.iter().collect::<Vec<_>>().join(",");
                        // Google uses specific header patterns
                        return headers_str.contains("Given Name")
                            || headers_str.contains("E-mail 1 - Value")
                            || headers_str.contains("Phone 1 - Value");
                    }
                }
            }
        }
        false
    }

    async fn parse(&self, file_path: &Path) -> Result<ParseResult, ImportError> {
        let mut reader = csv::Reader::from_path(file_path)?;
        let headers: Vec<String> = reader.headers()?.iter().map(|h| h.to_string()).collect();

        let mut rows = Vec::new();
        let mut warnings = Vec::new();

        for (idx, result) in reader.records().enumerate() {
            match result {
                Ok(record) => {
                    let mut row = HashMap::new();

                    for (i, field) in record.iter().enumerate() {
                        if let Some(header) = headers.get(i) {
                            // Map Google-specific headers to standard fields
                            let normalized_key = match header.as_str() {
                                "Given Name" => "first_name",
                                "Family Name" | "Additional Name"
                                    if row.get("last_name").is_none() =>
                                {
                                    "last_name"
                                }
                                "E-mail 1 - Value" | "E-mail 1 - Type * ::: E-mail 1 - Value" => {
                                    "email"
                                }
                                "Phone 1 - Value" | "Phone 1 - Type * ::: Phone 1 - Value" => {
                                    "phone"
                                }
                                "Organization 1 - Name" => "organization",
                                "Organization 1 - Title" => "title",
                                "Notes" => "notes",
                                "Address 1 - Formatted" => "address",
                                "Website 1 - Value" => "website",
                                "Birthday" => "birthday",
                                // Support alternate email/phone fields
                                h if h.starts_with("E-mail")
                                    && h.ends_with("Value")
                                    && !row.contains_key("email") =>
                                {
                                    "email"
                                }
                                h if h.starts_with("Phone")
                                    && h.ends_with("Value")
                                    && !row.contains_key("phone") =>
                                {
                                    "phone"
                                }
                                _ => header.as_str(),
                            };

                            if !field.is_empty() {
                                row.insert(normalized_key.to_string(), field.to_string());
                            }
                        }
                    }

                    if !row.is_empty()
                        && (row.contains_key("email") || row.contains_key("first_name"))
                    {
                        row.insert("source".to_string(), "google_contacts".to_string());
                        rows.push(row);
                    } else {
                        warnings.push(format!("Row {} missing required fields, skipped", idx + 1));
                    }
                }
                Err(e) => {
                    warnings.push(format!("CSV parsing error at row {}: {}", idx + 1, e));
                }
            }
        }

        let suggested_mappings = vec![
            ("first_name".to_string(), "first_name".to_string()),
            ("last_name".to_string(), "last_name".to_string()),
            ("email".to_string(), "email".to_string()),
            ("phone".to_string(), "phone".to_string()),
            ("organization".to_string(), "organization".to_string()),
            ("title".to_string(), "title".to_string()),
            ("notes".to_string(), "notes".to_string()),
        ];

        let mut metadata = HashMap::new();
        metadata.insert("format".to_string(), "google_contacts_csv".to_string());
        metadata.insert("total_contacts".to_string(), rows.len().to_string());

        Ok(ParseResult {
            rows,
            suggested_mappings,
            metadata,
            warnings,
        })
    }
}

/// Apple Contacts vCard Connector
pub struct AppleContactsConnector;

impl AppleContactsConnector {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ImportConnector for AppleContactsConnector {
    fn metadata(&self) -> ConnectorMetadata {
        ConnectorMetadata {
            id: "apple_contacts".to_string(),
            name: "Apple Contacts".to_string(),
            description: "Import contacts from Apple Contacts vCard (.vcf) export".to_string(),
            supported_extensions: vec!["vcf".to_string(), "vcard".to_string()],
            supported_mime_types: vec!["text/vcard".to_string(), "text/x-vcard".to_string()],
            format: ImportFormat::Vcard,
            required_auth_scopes: vec![],
            version: "1.0.0".to_string(),
        }
    }

    async fn parse(&self, file_path: &Path) -> Result<ParseResult, ImportError> {
        let content = std::fs::read_to_string(file_path)?;
        let mut rows = Vec::new();
        let warnings = Vec::new();
        let mut current = HashMap::new();

        for line in content.lines() {
            let line = line.trim();

            if line.starts_with("BEGIN:VCARD") {
                current = HashMap::new();
            } else if line.starts_with("END:VCARD") {
                if !current.is_empty() {
                    current.insert("source".to_string(), "apple_contacts".to_string());
                    rows.push(current.clone());
                }
                current = HashMap::new();
            } else if let Some((key, value)) = line.split_once(':') {
                let key_parts: Vec<&str> = key.split(';').collect();
                let base_key = key_parts[0];

                match base_key {
                    "N" => {
                        // N:Family;Given;Middle;Prefix;Suffix
                        let parts: Vec<&str> = value.split(';').collect();
                        if parts.len() >= 2 {
                            if !parts[0].is_empty() {
                                current.insert("last_name".to_string(), parts[0].to_string());
                            }
                            if !parts[1].is_empty() {
                                current.insert("first_name".to_string(), parts[1].to_string());
                            }
                        }
                    }
                    "FN" => {
                        // Full name - use if N not present
                        if !current.contains_key("first_name") {
                            let parts: Vec<&str> = value.split_whitespace().collect();
                            if let Some(first) = parts.first() {
                                current.insert("first_name".to_string(), first.to_string());
                            }
                            if parts.len() > 1 {
                                if let Some(last) = parts.last() {
                                    current.insert("last_name".to_string(), last.to_string());
                                }
                            }
                        }
                    }
                    "EMAIL" => {
                        if !current.contains_key("email") {
                            current.insert("email".to_string(), value.to_string());
                        }
                    }
                    "TEL" => {
                        if !current.contains_key("phone") {
                            current.insert("phone".to_string(), value.to_string());
                        }
                    }
                    "ORG" => {
                        current.insert("organization".to_string(), value.to_string());
                    }
                    "TITLE" => {
                        current.insert("title".to_string(), value.to_string());
                    }
                    "NOTE" => {
                        current.insert("notes".to_string(), value.to_string());
                    }
                    "ADR" => {
                        // ADR:;;Street;City;State;ZIP;Country
                        current.insert("address".to_string(), value.replace(';', ", "));
                    }
                    "URL" => {
                        current.insert("website".to_string(), value.to_string());
                    }
                    "BDAY" => {
                        current.insert("birthday".to_string(), value.to_string());
                    }
                    _ => {}
                }
            }
        }

        let suggested_mappings = vec![
            ("first_name".to_string(), "first_name".to_string()),
            ("last_name".to_string(), "last_name".to_string()),
            ("email".to_string(), "email".to_string()),
            ("phone".to_string(), "phone".to_string()),
            ("organization".to_string(), "organization".to_string()),
            ("title".to_string(), "title".to_string()),
            ("notes".to_string(), "notes".to_string()),
        ];

        let mut metadata = HashMap::new();
        metadata.insert("format".to_string(), "vcard".to_string());
        metadata.insert("total_contacts".to_string(), rows.len().to_string());

        Ok(ParseResult {
            rows,
            suggested_mappings,
            metadata,
            warnings,
        })
    }
}

/// Generic CSV Connector (fallback for other contact CSV formats)
pub struct GenericCsvConnector;

impl GenericCsvConnector {
    pub fn new() -> Self {
        Self
    }

    /// Attempt to auto-map common column names to standard fields
    fn normalize_header(header: &str) -> &str {
        let lower = header.to_lowercase();
        match lower.as_str() {
            h if h.contains("first") && h.contains("name") => "first_name",
            h if h.contains("last") && h.contains("name") => "last_name",
            h if h.contains("given") && h.contains("name") => "first_name",
            h if h.contains("family") && h.contains("name") => "last_name",
            h if h.contains("email") || h.contains("e-mail") => "email",
            h if h.contains("phone") || h.contains("mobile") || h.contains("cell") => "phone",
            h if h.contains("company") || h.contains("organization") => "organization",
            h if h.contains("title") || h.contains("job") => "title",
            h if h.contains("note") || h.contains("comment") => "notes",
            h if h.contains("address") || h.contains("street") => "address",
            h if h.contains("city") => "city",
            h if h.contains("state") || h.contains("province") => "state",
            h if h.contains("zip") || h.contains("postal") => "postal_code",
            h if h.contains("country") => "country",
            _ => header,
        }
    }
}

#[async_trait]
impl ImportConnector for GenericCsvConnector {
    fn metadata(&self) -> ConnectorMetadata {
        ConnectorMetadata {
            id: "generic_csv".to_string(),
            name: "Generic CSV".to_string(),
            description: "Import contacts from any CSV file with auto-detection".to_string(),
            supported_extensions: vec!["csv".to_string()],
            supported_mime_types: vec!["text/csv".to_string()],
            format: ImportFormat::Csv,
            required_auth_scopes: vec![],
            version: "1.0.0".to_string(),
        }
    }

    async fn parse(&self, file_path: &Path) -> Result<ParseResult, ImportError> {
        let mut reader = csv::Reader::from_path(file_path)?;
        let headers: Vec<String> = reader
            .headers()?
            .iter()
            .map(|h| Self::normalize_header(h).to_string())
            .collect();

        let mut rows = Vec::new();
        let mut warnings = Vec::new();

        for (idx, result) in reader.records().enumerate() {
            match result {
                Ok(record) => {
                    let mut row = HashMap::new();

                    for (i, field) in record.iter().enumerate() {
                        if let Some(header) = headers.get(i) {
                            if !field.is_empty() {
                                row.insert(header.clone(), field.to_string());
                            }
                        }
                    }

                    if !row.is_empty() {
                        row.insert("source".to_string(), "csv".to_string());
                        rows.push(row);
                    }
                }
                Err(e) => {
                    warnings.push(format!("CSV parsing error at row {}: {}", idx + 1, e));
                }
            }
        }

        // Generate suggested mappings based on detected fields
        let mut suggested_mappings = Vec::new();
        let standard_fields = [
            "first_name",
            "last_name",
            "email",
            "phone",
            "organization",
            "title",
            "notes",
        ];

        for field in standard_fields {
            if headers.iter().any(|h| h == field) {
                suggested_mappings.push((field.to_string(), field.to_string()));
            }
        }

        let mut metadata = HashMap::new();
        metadata.insert("format".to_string(), "csv".to_string());
        metadata.insert("total_contacts".to_string(), rows.len().to_string());
        metadata.insert("detected_fields".to_string(), headers.join(", "));

        Ok(ParseResult {
            rows,
            suggested_mappings,
            metadata,
            warnings,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_google_contacts_csv() {
        let csv_content =
            "Given Name,Family Name,E-mail 1 - Value,Phone 1 - Value,Organization 1 - Name\n\
John,Doe,john@example.com,555-1234,Acme Corp";

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_content).unwrap();

        let connector = GoogleContactsConnector::new();
        let result = connector.parse(temp_file.path()).await.unwrap();

        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0].get("first_name").unwrap(), "John");
        assert_eq!(result.rows[0].get("email").unwrap(), "john@example.com");
    }

    #[tokio::test]
    async fn test_apple_vcard() {
        let vcard_content = r#"BEGIN:VCARD
VERSION:3.0
N:Doe;John;;;
FN:John Doe
EMAIL:john@example.com
TEL:555-1234
ORG:Acme Corp
END:VCARD"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", vcard_content).unwrap();

        let connector = AppleContactsConnector::new();
        let result = connector.parse(temp_file.path()).await.unwrap();

        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0].get("first_name").unwrap(), "John");
        assert_eq!(result.rows[0].get("email").unwrap(), "john@example.com");
    }
}
