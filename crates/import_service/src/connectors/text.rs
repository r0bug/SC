use crate::{
    config::ImportFormat,
    connector::{ConnectorMetadata, ImportConnector, ParseResult},
    ImportError,
};
use async_trait::async_trait;
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;

/// Plain Text Import Connector
///
/// Supports:
/// - Tab-separated values
/// - Space-separated columns (aligned text)
/// - Key-value pairs (Name: John\nEmail: john@email.com)
/// - Simple lists (one contact per line)
pub struct TextImporter;

impl TextImporter {
    pub fn new() -> Self {
        Self
    }

    /// Detect the delimiter used in the text file
    fn detect_delimiter(lines: &[String]) -> (char, f32) {
        let delimiters = vec![
            ('\t', "tab"),
            (',', "comma"),
            (';', "semicolon"),
            ('|', "pipe"),
        ];
        let mut scores = Vec::new();

        for (delim, _name) in delimiters {
            let mut counts = Vec::new();
            for line in lines.iter().take(10) {
                if !line.trim().is_empty() {
                    counts.push(line.chars().filter(|&c| c == delim).count());
                }
            }

            if counts.is_empty() {
                continue;
            }

            // Check consistency - good delimiter should appear same number of times in each line
            let avg = counts.iter().sum::<usize>() as f32 / counts.len() as f32;
            let variance: f32 = counts
                .iter()
                .map(|&c| {
                    let diff = c as f32 - avg;
                    diff * diff
                })
                .sum::<f32>()
                / counts.len() as f32;

            // Score: higher average count, lower variance is better
            let score = if variance < 1.0 {
                avg
            } else {
                avg / variance.sqrt()
            };

            scores.push((delim, score));
        }

        // Return delimiter with highest score
        scores
            .into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap_or(('\t', 0.0))
    }

    /// Check if text looks like key-value format
    fn is_key_value_format(lines: &[String]) -> bool {
        let kv_pattern = Regex::new(r"^[A-Za-z\s]+:\s*.+$").unwrap();
        let matching_lines = lines
            .iter()
            .take(10)
            .filter(|line| !line.trim().is_empty())
            .filter(|line| kv_pattern.is_match(line))
            .count();

        matching_lines >= 3 // At least 3 key:value lines
    }

    /// Parse key-value format
    fn parse_key_value(content: &str) -> Vec<HashMap<String, String>> {
        let lines: Vec<&str> = content.lines().collect();
        let mut rows = Vec::new();
        let mut current_record = HashMap::new();

        for line in lines {
            let trimmed = line.trim();

            // Empty line might indicate new record
            if trimmed.is_empty() {
                if !current_record.is_empty() {
                    rows.push(current_record.clone());
                    current_record.clear();
                }
                continue;
            }

            // Parse key:value
            if let Some((key, value)) = trimmed.split_once(':') {
                let key = key.trim().to_lowercase().replace(' ', "_");
                let value = value.trim().to_string();

                if !value.is_empty() {
                    current_record.insert(key, value);
                }
            }
        }

        // Don't forget last record
        if !current_record.is_empty() {
            rows.push(current_record);
        }

        rows
    }

    /// Parse delimited format
    fn parse_delimited(content: &str, delimiter: char) -> Vec<HashMap<String, String>> {
        let lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();

        if lines.is_empty() {
            return vec![];
        }

        // First line as headers
        let headers: Vec<String> = lines[0]
            .split(delimiter)
            .map(|h| h.trim().to_string())
            .collect();

        if headers.is_empty() {
            return vec![];
        }

        let mut rows = Vec::new();

        for line in lines.iter().skip(1) {
            if line.trim().is_empty() {
                continue;
            }

            let values: Vec<String> = line
                .split(delimiter)
                .map(|v| v.trim().to_string())
                .collect();

            let mut record = HashMap::new();
            for (i, value) in values.iter().enumerate() {
                if value.is_empty() {
                    continue;
                }

                let key = if i < headers.len() && !headers[i].is_empty() {
                    headers[i].clone()
                } else {
                    format!("column_{}", i + 1)
                };

                record.insert(key, value.clone());
            }

            if !record.is_empty() {
                rows.push(record);
            }
        }

        rows
    }

    /// Parse aligned columnar text (space-separated with alignment)
    fn parse_columnar(content: &str) -> Vec<HashMap<String, String>> {
        let lines: Vec<String> = content
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| l.to_string())
            .collect();

        if lines.len() < 2 {
            return vec![];
        }

        // Detect column positions from first line (assumed to be header)
        let header_line = &lines[0];
        let headers: Vec<(usize, usize, String)> = Self::detect_column_positions(header_line);

        if headers.is_empty() {
            return vec![];
        }

        let mut rows = Vec::new();

        for line in lines.iter().skip(1) {
            let mut record = HashMap::new();

            for (start, end, name) in &headers {
                if line.len() < *end {
                    continue;
                }

                let value = if line.len() >= *end {
                    &line[*start..*end]
                } else {
                    &line[*start..]
                };

                let trimmed = value.trim().to_string();
                if !trimmed.is_empty() {
                    record.insert(name.clone(), trimmed);
                }
            }

            if !record.is_empty() {
                rows.push(record);
            }
        }

        rows
    }

    /// Detect column positions in aligned text
    fn detect_column_positions(line: &str) -> Vec<(usize, usize, String)> {
        let words: Vec<(usize, &str)> = line
            .split_whitespace()
            .scan(0, |pos, word| {
                let start = line[*pos..].find(word).map(|i| *pos + i)?;
                *pos = start + word.len();
                Some((start, word))
            })
            .collect();

        let mut columns = Vec::new();
        for i in 0..words.len() {
            let (start, word) = words[i];
            let end = if i + 1 < words.len() {
                words[i + 1].0
            } else {
                line.len()
            };

            columns.push((start, end, word.to_string()));
        }

        columns
    }

    /// Auto-detect field names from sample records
    fn detect_fields(records: &[HashMap<String, String>]) -> Vec<String> {
        let mut field_set = std::collections::HashSet::new();

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
                _ if lower == "lastname" || lower == "lname" || lower == "surname" => {
                    Some("last_name")
                }
                _ if lower == "name" => Some("first_name"),
                _ if lower.contains("email") || lower == "e-mail" || lower == "mail" => {
                    Some("email")
                }
                _ if lower.contains("phone") || lower.contains("mobile") => Some("phone"),
                _ if lower.contains("company") || lower.contains("organization") => {
                    Some("organization")
                }
                _ if lower.contains("title") || lower.contains("position") => Some("title"),
                _ if lower.contains("note") || lower.contains("comment") => Some("notes"),
                _ if lower.contains("address") => Some("address"),
                _ if lower == "city" => Some("city"),
                _ if lower == "state" => Some("state"),
                _ if lower.contains("zip") || lower.contains("postal") => Some("postal_code"),
                _ if lower == "country" => Some("country"),
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
impl ImportConnector for TextImporter {
    fn metadata(&self) -> ConnectorMetadata {
        ConnectorMetadata {
            id: "text".to_string(),
            name: "Plain Text File".to_string(),
            description:
                "Import from plain text files with tab/space delimiters or key-value format"
                    .to_string(),
            supported_extensions: vec!["txt".to_string(), "text".to_string()],
            supported_mime_types: vec!["text/plain".to_string()],
            format: ImportFormat::Text,
            required_auth_scopes: vec![],
            version: "1.0.0".to_string(),
        }
    }

    async fn parse(&self, file_path: &Path) -> Result<ParseResult, ImportError> {
        let content = std::fs::read_to_string(file_path)?;
        let lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();

        if lines.is_empty() {
            return Err(ImportError::Other("Empty file".to_string()));
        }

        // Detect format and parse
        let (rows, format_type) = if Self::is_key_value_format(&lines) {
            (Self::parse_key_value(&content), "key_value".to_string())
        } else {
            let (delimiter, score) = Self::detect_delimiter(&lines);

            if score > 0.5 {
                (
                    Self::parse_delimited(&content, delimiter),
                    format!("delimited:{}", delimiter),
                )
            } else {
                // Try columnar format
                let columnar_rows = Self::parse_columnar(&content);
                if !columnar_rows.is_empty() {
                    (columnar_rows, "columnar".to_string())
                } else {
                    return Err(ImportError::Other(
                        "Could not detect text format. Expected delimited, key-value, or columnar data.".to_string(),
                    ));
                }
            }
        };

        if rows.is_empty() {
            return Err(ImportError::Other(
                "No valid records found in text file".to_string(),
            ));
        }

        // Add source to all rows
        let mut rows = rows;
        for row in &mut rows {
            row.insert("source".to_string(), "text".to_string());
        }

        // Detect fields and generate mappings
        let detected_fields = Self::detect_fields(&rows);
        let suggested_mappings = Self::suggest_mappings(&detected_fields);

        // Build metadata
        let mut metadata = HashMap::new();
        metadata.insert("format".to_string(), "text".to_string());
        metadata.insert("text_format".to_string(), format_type);
        metadata.insert("total_records".to_string(), rows.len().to_string());
        metadata.insert("detected_fields".to_string(), detected_fields.join(", "));

        Ok(ParseResult {
            rows,
            suggested_mappings,
            metadata,
            warnings: vec![],
        })
    }
}

impl Default for TextImporter {
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
    async fn test_tab_delimited() {
        let text_content = "First Name\tLast Name\tEmail\tPhone\nJohn\tDoe\tjohn@example.com\t555-1234\nJane\tSmith\tjane@example.com\t555-5678";

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", text_content).unwrap();

        let connector = TextImporter::new();
        let result = connector.parse(temp_file.path()).await.unwrap();

        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.rows[0].get("First Name").unwrap(), "John");
        assert_eq!(result.rows[1].get("First Name").unwrap(), "Jane");
        assert!(result
            .metadata
            .get("text_format")
            .unwrap()
            .contains("delimited"));
    }

    #[tokio::test]
    async fn test_key_value_format() {
        let text_content = r#"First Name: John
Last Name: Doe
Email: john@example.com
Phone: 555-1234

First Name: Jane
Last Name: Smith
Email: jane@example.com"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", text_content).unwrap();

        let connector = TextImporter::new();
        let result = connector.parse(temp_file.path()).await.unwrap();

        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.rows[0].get("first_name").unwrap(), "John");
        assert_eq!(result.rows[1].get("first_name").unwrap(), "Jane");
        assert_eq!(result.metadata.get("text_format").unwrap(), "key_value");
    }

    #[tokio::test]
    async fn test_columnar_format() {
        let text_content = r#"FirstName  LastName   Email                Phone
John       Doe        john@example.com     555-1234
Jane       Smith      jane@example.com     555-5678"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", text_content).unwrap();

        let connector = TextImporter::new();
        let result = connector.parse(temp_file.path()).await.unwrap();

        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.rows[0].get("FirstName").unwrap(), "John");
        assert_eq!(result.rows[1].get("FirstName").unwrap(), "Jane");
        assert_eq!(result.metadata.get("text_format").unwrap(), "columnar");
    }

    #[tokio::test]
    async fn test_field_mapping_suggestions() {
        let text_content = "fname\tlname\tmail\tmobile\nJohn\tDoe\tjohn@example.com\t555-1234";

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", text_content).unwrap();

        let connector = TextImporter::new();
        let result = connector.parse(temp_file.path()).await.unwrap();

        let mappings: HashMap<_, _> = result.suggested_mappings.into_iter().collect();
        assert_eq!(mappings.get("fname"), Some(&"first_name".to_string()));
        assert_eq!(mappings.get("lname"), Some(&"last_name".to_string()));
        assert_eq!(mappings.get("mail"), Some(&"email".to_string()));
        assert_eq!(mappings.get("mobile"), Some(&"phone".to_string()));
    }
}
