use crate::{
    config::ImportFormat,
    connector::{ConnectorMetadata, ImportConnector, ParseResult},
    ImportError,
};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

/// JSON File Import Connector
///
/// Supports:
/// - Simple array format: `[{contact1}, {contact2}]`
/// - Nested format: `{"contacts": [{}, {}], "metadata": {}}`
/// - Deeply nested structures with flattening
pub struct JsonImporter;

impl JsonImporter {
    pub fn new() -> Self {
        Self
    }

    /// Flatten nested JSON objects using dot notation
    /// Example: {"user": {"name": "John"}} → {"user.name": "John"}
    fn flatten_object(obj: &Value, prefix: &str, result: &mut HashMap<String, String>) {
        match obj {
            Value::Object(map) => {
                for (key, value) in map {
                    let new_key = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", prefix, key)
                    };

                    match value {
                        Value::Object(_) => {
                            Self::flatten_object(value, &new_key, result);
                        }
                        Value::Array(arr) => {
                            // For arrays, stringify if simple values, or flatten first item as sample
                            if arr.is_empty() {
                                continue;
                            }
                            if matches!(arr[0], Value::Object(_)) {
                                // Array of objects - take first as sample
                                Self::flatten_object(&arr[0], &new_key, result);
                            } else {
                                // Array of primitives - join as string
                                let values: Vec<String> = arr
                                    .iter()
                                    .filter_map(|v| Self::value_to_string(v))
                                    .collect();
                                if !values.is_empty() {
                                    result.insert(new_key, values.join(", "));
                                }
                            }
                        }
                        _ => {
                            if let Some(val_str) = Self::value_to_string(value) {
                                result.insert(new_key, val_str);
                            }
                        }
                    }
                }
            }
            _ => {
                if let Some(val_str) = Self::value_to_string(obj) {
                    result.insert(prefix.to_string(), val_str);
                }
            }
        }
    }

    /// Convert JSON value to string, handling all types
    fn value_to_string(value: &Value) -> Option<String> {
        match value {
            Value::String(s) => Some(s.clone()),
            Value::Number(n) => Some(n.to_string()),
            Value::Bool(b) => Some(b.to_string()),
            Value::Null => None,
            Value::Array(_) | Value::Object(_) => Some(value.to_string()),
        }
    }

    /// Detect the structure of the JSON and extract records
    /// Returns (records, detected_structure_info)
    fn extract_records(value: Value) -> Result<(Vec<Value>, String), ImportError> {
        match value {
            Value::Array(arr) => {
                // Simple array of contacts
                Ok((arr, "simple_array".to_string()))
            }
            Value::Object(map) => {
                // Look for array fields that might contain contacts
                let contact_keys = ["contacts", "data", "items", "results", "people", "users"];

                for key in &contact_keys {
                    if let Some(Value::Array(arr)) = map.get(*key) {
                        return Ok((arr.clone(), format!("nested_array:{}", key)));
                    }
                }

                // If no array found, treat the single object as one record
                Ok((vec![Value::Object(map)], "single_object".to_string()))
            }
            _ => Err(ImportError::Other(
                "JSON must be an object or array".to_string(),
            )),
        }
    }

    /// Auto-detect field names from sample records
    fn detect_fields(records: &[HashMap<String, String>]) -> Vec<String> {
        let mut field_set = std::collections::HashSet::new();

        // Collect all unique fields from first 10 records
        for record in records.iter().take(10) {
            for key in record.keys() {
                field_set.insert(key.clone());
            }
        }

        let mut fields: Vec<String> = field_set.into_iter().collect();
        fields.sort();
        fields
    }

    /// Generate suggested mappings based on field names
    fn suggest_mappings(fields: &[String]) -> Vec<(String, String)> {
        let mut mappings = Vec::new();

        for field in fields {
            let lower = field.to_lowercase();
            let suggested_target = match () {
                _ if lower.contains("first") && lower.contains("name") => Some("first_name"),
                _ if lower == "firstname" || lower == "fname" || lower == "given_name" => {
                    Some("first_name")
                }
                _ if lower.contains("last") && lower.contains("name") => Some("last_name"),
                _ if lower == "lastname" || lower == "lname" || lower == "surname" || lower == "family_name" => {
                    Some("last_name")
                }
                _ if lower == "name" || lower == "full_name" || lower == "fullname" => {
                    Some("first_name")
                }
                _ if lower.contains("email") || lower == "mail" || lower == "e-mail" => {
                    Some("email")
                }
                _ if lower.contains("phone") || lower.contains("mobile") || lower.contains("cell") => {
                    Some("phone")
                }
                _ if lower.contains("company") || lower.contains("organization") || lower == "org" => {
                    Some("organization")
                }
                _ if lower.contains("title") || lower.contains("position") || lower.contains("job") => {
                    Some("title")
                }
                _ if lower.contains("note") || lower.contains("comment") || lower.contains("description") => {
                    Some("notes")
                }
                _ if lower.contains("address") || lower.contains("street") => Some("address"),
                _ if lower == "city" => Some("city"),
                _ if lower == "state" || lower == "province" || lower == "region" => Some("state"),
                _ if lower.contains("zip") || lower.contains("postal") => Some("postal_code"),
                _ if lower == "country" => Some("country"),
                _ if lower.contains("website") || lower.contains("url") || lower == "homepage" => {
                    Some("website")
                }
                _ if lower.contains("birthday") || lower == "dob" || lower == "birth_date" => {
                    Some("birthday")
                }
                _ => None,
            };

            if let Some(target) = suggested_target {
                mappings.push((field.clone(), target.to_string()));
            }
        }

        mappings
    }
}

#[async_trait]
impl ImportConnector for JsonImporter {
    fn metadata(&self) -> ConnectorMetadata {
        ConnectorMetadata {
            id: "json".to_string(),
            name: "JSON File".to_string(),
            description: "Import from JSON files with arrays of contact objects or nested structures".to_string(),
            supported_extensions: vec!["json".to_string()],
            supported_mime_types: vec!["application/json".to_string()],
            format: ImportFormat::Json,
            required_auth_scopes: vec![],
            version: "1.0.0".to_string(),
        }
    }

    async fn parse(&self, file_path: &Path) -> Result<ParseResult, ImportError> {
        // Read and parse JSON file
        let content = std::fs::read_to_string(file_path)?;
        let json_value: Value = serde_json::from_str(&content).map_err(|e| {
            ImportError::Other(format!("Invalid JSON: {}", e))
        })?;

        // Extract records from JSON structure
        let (records, structure_info) = Self::extract_records(json_value)?;

        // Flatten each record to HashMap<String, String>
        let mut rows = Vec::new();
        let mut warnings = Vec::new();

        for (idx, record) in records.iter().enumerate() {
            let mut row = HashMap::new();
            Self::flatten_object(record, "", &mut row);

            if row.is_empty() {
                warnings.push(format!("Record {} is empty, skipped", idx + 1));
                continue;
            }

            row.insert("source".to_string(), "json".to_string());
            rows.push(row);
        }

        if rows.is_empty() {
            return Err(ImportError::Other(
                "No valid records found in JSON file".to_string(),
            ));
        }

        // Detect fields and generate mappings
        let detected_fields = Self::detect_fields(&rows);
        let suggested_mappings = Self::suggest_mappings(&detected_fields);

        // Build metadata
        let mut metadata = HashMap::new();
        metadata.insert("format".to_string(), "json".to_string());
        metadata.insert("structure".to_string(), structure_info);
        metadata.insert("total_records".to_string(), rows.len().to_string());
        metadata.insert("detected_fields".to_string(), detected_fields.join(", "));
        metadata.insert("nested_support".to_string(), "true".to_string());

        Ok(ParseResult {
            rows,
            suggested_mappings,
            metadata,
            warnings,
        })
    }
}

impl Default for JsonImporter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_simple_array() {
        let json_content = r#"[
            {
                "first_name": "John",
                "last_name": "Doe",
                "email": "john@example.com",
                "phone": "555-1234"
            },
            {
                "first_name": "Jane",
                "last_name": "Smith",
                "email": "jane@example.com"
            }
        ]"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", json_content).unwrap();

        let connector = JsonImporter::new();
        let result = connector.parse(temp_file.path()).await.unwrap();

        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.rows[0].get("first_name").unwrap(), "John");
        assert_eq!(result.rows[1].get("first_name").unwrap(), "Jane");
        assert!(result.suggested_mappings.len() > 0);
    }

    #[tokio::test]
    async fn test_nested_structure() {
        let json_content = r#"{
            "contacts": [
                {
                    "name": {"first": "John", "last": "Doe"},
                    "contact_info": {
                        "email": "john@example.com",
                        "phone": "555-1234"
                    }
                }
            ],
            "metadata": {
                "export_date": "2024-01-01"
            }
        }"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", json_content).unwrap();

        let connector = JsonImporter::new();
        let result = connector.parse(temp_file.path()).await.unwrap();

        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0].get("name.first").unwrap(), "John");
        assert_eq!(result.rows[0].get("contact_info.email").unwrap(), "john@example.com");
        assert_eq!(result.metadata.get("structure").unwrap(), "nested_array:contacts");
    }

    #[tokio::test]
    async fn test_single_object() {
        let json_content = r#"{
            "firstName": "John",
            "lastName": "Doe",
            "email": "john@example.com"
        }"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", json_content).unwrap();

        let connector = JsonImporter::new();
        let result = connector.parse(temp_file.path()).await.unwrap();

        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0].get("firstName").unwrap(), "John");
        assert_eq!(result.metadata.get("structure").unwrap(), "single_object");
    }

    #[tokio::test]
    async fn test_field_mapping_suggestions() {
        let json_content = r#"[{
            "fname": "John",
            "lname": "Doe",
            "mail": "john@example.com",
            "mobile": "555-1234",
            "company": "Acme Corp"
        }]"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", json_content).unwrap();

        let connector = JsonImporter::new();
        let result = connector.parse(temp_file.path()).await.unwrap();

        // Check that auto-mapping detected variations
        let mappings: HashMap<_, _> = result.suggested_mappings.into_iter().collect();
        assert_eq!(mappings.get("fname"), Some(&"first_name".to_string()));
        assert_eq!(mappings.get("lname"), Some(&"last_name".to_string()));
        assert_eq!(mappings.get("mail"), Some(&"email".to_string()));
        assert_eq!(mappings.get("mobile"), Some(&"phone".to_string()));
        assert_eq!(mappings.get("company"), Some(&"organization".to_string()));
    }
}
