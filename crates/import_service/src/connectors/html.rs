use crate::{
    config::ImportFormat,
    connector::{ConnectorMetadata, ImportConnector, ParseResult},
    ImportError,
};
use async_trait::async_trait;
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::path::Path;

/// HTML Import Connector
///
/// Supports:
/// - HTML tables (most common export format)
/// - Structured lists with nested data
/// - Divs with class/id patterns (e.g., contact cards)
/// - Common export formats (LinkedIn, Gmail, etc.)
pub struct HtmlImporter;

impl HtmlImporter {
    pub fn new() -> Self {
        Self
    }

    /// Extract data from HTML tables
    fn parse_table(table: scraper::ElementRef) -> Vec<HashMap<String, String>> {
        let mut rows = Vec::new();
        let mut headers = Vec::new();

        // Try to find headers in <thead> or first <tr>
        let _thead_selector = Selector::parse("thead tr th, thead tr td").unwrap();
        let tr_selector = Selector::parse("tr").unwrap();
        let td_selector = Selector::parse("td, th").unwrap();

        // Check for thead
        if let Some(thead_row) = table.select(&Selector::parse("thead tr").unwrap()).next() {
            headers = thead_row
                .select(&Selector::parse("th, td").unwrap())
                .map(|cell| cell.text().collect::<String>().trim().to_string())
                .collect();
        } else {
            // Try first row as header
            if let Some(first_row) = table.select(&tr_selector).next() {
                let cells: Vec<String> = first_row
                    .select(&td_selector)
                    .map(|cell| cell.text().collect::<String>().trim().to_string())
                    .collect();

                // Check if first row looks like headers (all non-empty, no numbers)
                let looks_like_header = cells.iter().all(|c| {
                    !c.is_empty() && !c.chars().all(|ch| ch.is_numeric() || ch.is_whitespace())
                });

                if looks_like_header {
                    headers = cells;
                }
            }
        }

        // Parse data rows (skip first if it was used as header)
        let has_thead = table
            .select(&Selector::parse("thead").unwrap())
            .next()
            .is_some();

        let skip_first = !headers.is_empty() && !has_thead;

        // Use tbody rows if available, otherwise all rows
        let data_row_selector = if has_thead {
            Selector::parse("tbody tr").unwrap()
        } else {
            tr_selector.clone()
        };

        for (idx, row) in table.select(&data_row_selector).enumerate() {
            if skip_first && idx == 0 {
                continue;
            }

            let cells: Vec<String> = row
                .select(&td_selector)
                .map(|cell| cell.text().collect::<String>().trim().to_string())
                .collect();

            if cells.is_empty() || cells.iter().all(|c| c.is_empty()) {
                continue;
            }

            let mut record = HashMap::new();

            for (i, cell_value) in cells.iter().enumerate() {
                if cell_value.is_empty() {
                    continue;
                }

                let key = if i < headers.len() && !headers[i].is_empty() {
                    headers[i].clone()
                } else {
                    format!("column_{}", i + 1)
                };

                record.insert(key, cell_value.clone());
            }

            if !record.is_empty() {
                rows.push(record);
            }
        }

        rows
    }

    /// Extract data from div-based contact cards
    fn parse_contact_cards(document: &Html) -> Vec<HashMap<String, String>> {
        let mut rows = Vec::new();

        // Common class patterns for contact cards
        let patterns = [
            ".contact",
            ".contact-card",
            ".person",
            ".profile",
            "[class*='contact']",
            "[class*='person']",
        ];

        for pattern in &patterns {
            if let Ok(selector) = Selector::parse(pattern) {
                for card in document.select(&selector) {
                    let mut record = HashMap::new();

                    // Try to extract common fields
                    let field_selectors = vec![
                        ("name", vec!["h1", "h2", "h3", ".name", "[class*='name']"]),
                        (
                            "email",
                            vec!["a[href^='mailto:']", ".email", "[class*='email']"],
                        ),
                        (
                            "phone",
                            vec!["a[href^='tel:']", ".phone", "[class*='phone']"],
                        ),
                        ("title", vec![".title", ".job-title", "[class*='title']"]),
                        (
                            "company",
                            vec![".company", ".organization", "[class*='company']"],
                        ),
                    ];

                    for (field_name, selectors) in field_selectors {
                        for sel_str in selectors {
                            if let Ok(sel) = Selector::parse(sel_str) {
                                if let Some(element) = card.select(&sel).next() {
                                    let text =
                                        element.text().collect::<String>().trim().to_string();
                                    if !text.is_empty() {
                                        // For mailto and tel links, extract the actual value
                                        let value = if sel_str.starts_with("a[href^='mailto:']") {
                                            element
                                                .value()
                                                .attr("href")
                                                .and_then(|h| h.strip_prefix("mailto:"))
                                                .unwrap_or(&text)
                                                .to_string()
                                        } else if sel_str.starts_with("a[href^='tel:']") {
                                            element
                                                .value()
                                                .attr("href")
                                                .and_then(|h| h.strip_prefix("tel:"))
                                                .unwrap_or(&text)
                                                .to_string()
                                        } else {
                                            text
                                        };

                                        record.insert(field_name.to_string(), value);
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    if !record.is_empty() {
                        rows.push(record);
                    }
                }

                if !rows.is_empty() {
                    break;
                }
            }
        }

        rows
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
                _ if lower == "firstname" || lower == "fname" || lower == "given name" => {
                    Some("first_name")
                }
                _ if lower.contains("last") && lower.contains("name") => Some("last_name"),
                _ if lower == "lastname"
                    || lower == "lname"
                    || lower == "surname"
                    || lower == "family name" =>
                {
                    Some("last_name")
                }
                _ if lower == "name" || lower == "full name" || lower == "fullname" => {
                    Some("first_name")
                }
                _ if lower.contains("email") || lower == "e-mail" || lower == "mail" => {
                    Some("email")
                }
                _ if lower.contains("phone")
                    || lower.contains("mobile")
                    || lower.contains("cell")
                    || lower == "telephone" =>
                {
                    Some("phone")
                }
                _ if lower.contains("company")
                    || lower.contains("organization")
                    || lower == "org" =>
                {
                    Some("organization")
                }
                _ if lower.contains("title")
                    || lower.contains("position")
                    || lower.contains("job") =>
                {
                    Some("title")
                }
                _ if lower.contains("note")
                    || lower.contains("comment")
                    || lower.contains("description") =>
                {
                    Some("notes")
                }
                _ if lower.contains("address") || lower.contains("street") => Some("address"),
                _ if lower == "city" => Some("city"),
                _ if lower == "state" || lower == "province" => Some("state"),
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
impl ImportConnector for HtmlImporter {
    fn metadata(&self) -> ConnectorMetadata {
        ConnectorMetadata {
            id: "html".to_string(),
            name: "HTML File".to_string(),
            description: "Import from HTML files with tables or structured contact cards"
                .to_string(),
            supported_extensions: vec!["html".to_string(), "htm".to_string()],
            supported_mime_types: vec!["text/html".to_string()],
            format: ImportFormat::Html,
            required_auth_scopes: vec![],
            version: "1.0.0".to_string(),
        }
    }

    async fn parse(&self, file_path: &Path) -> Result<ParseResult, ImportError> {
        let content = std::fs::read_to_string(file_path)?;
        let document = Html::parse_document(&content);

        // Try tables first (most common)
        let table_selector = Selector::parse("table").unwrap();
        let mut rows = Vec::new();
        let mut parse_method = String::new();

        for table in document.select(&table_selector) {
            let table_rows = Self::parse_table(table);
            if !table_rows.is_empty() {
                rows = table_rows;
                parse_method = "table".to_string();
                break;
            }
        }

        // If no tables found, try contact cards
        if rows.is_empty() {
            rows = Self::parse_contact_cards(&document);
            if !rows.is_empty() {
                parse_method = "contact_cards".to_string();
            }
        }

        if rows.is_empty() {
            return Err(ImportError::Other(
                "No structured data found in HTML file. Expected tables or contact cards."
                    .to_string(),
            ));
        }

        // Add source to all rows
        for row in &mut rows {
            row.insert("source".to_string(), "html".to_string());
        }

        // Detect fields and generate mappings
        let detected_fields = Self::detect_fields(&rows);
        let suggested_mappings = Self::suggest_mappings(&detected_fields);

        // Build metadata
        let mut metadata = HashMap::new();
        metadata.insert("format".to_string(), "html".to_string());
        metadata.insert("parse_method".to_string(), parse_method);
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

impl Default for HtmlImporter {
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
    async fn test_html_table() {
        let html_content = r#"<!DOCTYPE html>
<html>
<head><title>Contacts</title></head>
<body>
    <table>
        <thead>
            <tr>
                <th>First Name</th>
                <th>Last Name</th>
                <th>Email</th>
                <th>Phone</th>
            </tr>
        </thead>
        <tbody>
            <tr>
                <td>John</td>
                <td>Doe</td>
                <td>john@example.com</td>
                <td>555-1234</td>
            </tr>
            <tr>
                <td>Jane</td>
                <td>Smith</td>
                <td>jane@example.com</td>
                <td>555-5678</td>
            </tr>
        </tbody>
    </table>
</body>
</html>"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", html_content).unwrap();

        let connector = HtmlImporter::new();
        let result = connector.parse(temp_file.path()).await.unwrap();

        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.rows[0].get("First Name").unwrap(), "John");
        assert_eq!(result.rows[1].get("First Name").unwrap(), "Jane");
        assert_eq!(result.metadata.get("parse_method").unwrap(), "table");
    }

    #[tokio::test]
    async fn test_html_table_no_thead() {
        let html_content = r#"<table>
    <tr>
        <td>First Name</td>
        <td>Last Name</td>
        <td>Email</td>
    </tr>
    <tr>
        <td>John</td>
        <td>Doe</td>
        <td>john@example.com</td>
    </tr>
</table>"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", html_content).unwrap();

        let connector = HtmlImporter::new();
        let result = connector.parse(temp_file.path()).await.unwrap();

        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0].get("First Name").unwrap(), "John");
    }

    #[tokio::test]
    async fn test_contact_cards() {
        let html_content = r#"<!DOCTYPE html>
<html>
<body>
    <div class="contact">
        <h2 class="name">John Doe</h2>
        <div class="email">john@example.com</div>
        <div class="phone">555-1234</div>
        <div class="company">Acme Corp</div>
    </div>
    <div class="contact">
        <h2 class="name">Jane Smith</h2>
        <div class="email">jane@example.com</div>
    </div>
</body>
</html>"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", html_content).unwrap();

        let connector = HtmlImporter::new();
        let result = connector.parse(temp_file.path()).await.unwrap();

        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.rows[0].get("name").unwrap(), "John Doe");
        assert_eq!(result.rows[0].get("email").unwrap(), "john@example.com");
        assert_eq!(
            result.metadata.get("parse_method").unwrap(),
            "contact_cards"
        );
    }
}
