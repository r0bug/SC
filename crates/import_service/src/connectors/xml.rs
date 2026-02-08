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

/// Generic XML Import Connector
///
/// Supports:
/// - Repeating element detection (finds patterns like <contact>, <person>, <entry>)
/// - Attributes and element values
/// - Namespace handling
/// - Nested structures with dot notation
pub struct XmlImporter;

impl XmlImporter {
    pub fn new() -> Self {
        Self
    }

    /// Detect the repeating element pattern (likely the record element)
    fn detect_record_pattern(content: &str) -> Result<String, ImportError> {
        let mut reader = Reader::from_str(content);
        reader.trim_text(true);

        let mut element_counts: HashMap<String, usize> = HashMap::new();
        let mut buf = Vec::new();
        let mut depth = 0;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    depth += 1;
                    if depth == 2 {
                        // Count top-level child elements
                        let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                        *element_counts.entry(name).or_insert(0) += 1;
                    }
                }
                Ok(Event::End(_)) => {
                    depth -= 1;
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(ImportError::Other(format!("XML parse error: {}", e))),
                _ => {}
            }
            buf.clear();
        }

        // Find the element that appears most frequently (likely the record)
        let record_element = element_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(name, _)| name.clone())
            .ok_or_else(|| ImportError::Other("No repeating elements found in XML".to_string()))?;

        Ok(record_element)
    }

    /// Parse a single XML record element
    fn parse_record(
        reader: &mut Reader<&[u8]>,
        record_tag: &[u8],
        buf: &mut Vec<u8>,
    ) -> Result<HashMap<String, String>, ImportError> {
        let mut record = HashMap::new();
        let mut current_path = Vec::new();
        let mut current_text = String::new();

        loop {
            match reader.read_event_into(buf) {
                Ok(Event::Start(e)) => {
                    let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                    // Capture attributes
                    for attr_result in e.attributes() {
                        if let Ok(attr) = attr_result {
                            let attr_name = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            let attr_value = String::from_utf8_lossy(&attr.value).to_string();

                            let full_path = if current_path.is_empty() {
                                format!("{}@{}", tag_name, attr_name)
                            } else {
                                format!("{}.{}@{}", current_path.join("."), tag_name, attr_name)
                            };

                            if !attr_value.is_empty() {
                                record.insert(full_path, attr_value);
                            }
                        }
                    }

                    current_path.push(tag_name);
                    current_text.clear();
                }
                Ok(Event::Empty(e)) => {
                    // Handle self-closing tags like <contact email="..." />
                    let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                    // Capture attributes
                    for attr_result in e.attributes() {
                        if let Ok(attr) = attr_result {
                            let attr_name = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            let attr_value = String::from_utf8_lossy(&attr.value).to_string();

                            let full_path = if current_path.is_empty() {
                                format!("{}@{}", tag_name, attr_name)
                            } else {
                                format!("{}.{}@{}", current_path.join("."), tag_name, attr_name)
                            };

                            if !attr_value.is_empty() {
                                record.insert(full_path, attr_value);
                            }
                        }
                    }
                    // Note: Don't push to current_path since this is a self-closing tag
                }
                Ok(Event::End(e)) => {
                    let _tag_name = String::from_utf8_lossy(e.name().as_ref());

                    // Save element text if any
                    if !current_text.trim().is_empty() {
                        let path = current_path.join(".");
                        record.insert(path, current_text.trim().to_string());
                    }

                    current_path.pop();
                    current_text.clear();

                    // Check if we're done with this record
                    if e.name().as_ref() == record_tag {
                        break;
                    }
                }
                Ok(Event::Text(e)) => {
                    let text = e.unescape().unwrap_or_default();
                    current_text.push_str(&text);
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(ImportError::Other(format!(
                        "Error parsing XML record: {}",
                        e
                    )))
                }
                _ => {}
            }
            buf.clear();
        }

        Ok(record)
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
            // Remove XML path prefixes for better matching
            let field_lower = field.to_lowercase();
            let last_part = field.split('.').last().unwrap_or(field).to_lowercase();

            let suggested_target = match () {
                _ if last_part.contains("first") && last_part.contains("name") => {
                    Some("first_name")
                }
                _ if last_part == "firstname" || last_part == "fname" || last_part == "given" => {
                    Some("first_name")
                }
                _ if last_part.contains("last") && last_part.contains("name") => Some("last_name"),
                _ if last_part == "lastname"
                    || last_part == "lname"
                    || last_part == "surname"
                    || last_part == "family" =>
                {
                    Some("last_name")
                }
                _ if last_part == "name"
                    && !field_lower.contains("first")
                    && !field_lower.contains("last") =>
                {
                    Some("first_name")
                }
                _ if last_part.contains("email") || last_part == "mail" => Some("email"),
                _ if last_part.contains("phone")
                    || last_part.contains("mobile")
                    || last_part.contains("tel") =>
                {
                    Some("phone")
                }
                _ if last_part.contains("company")
                    || last_part.contains("organization")
                    || last_part == "org" =>
                {
                    Some("organization")
                }
                _ if last_part.contains("title")
                    || last_part.contains("position")
                    || last_part.contains("job") =>
                {
                    Some("title")
                }
                _ if last_part.contains("note")
                    || last_part.contains("comment")
                    || last_part.contains("description") =>
                {
                    Some("notes")
                }
                _ if last_part.contains("address") || last_part.contains("street") => {
                    Some("address")
                }
                _ if last_part == "city" => Some("city"),
                _ if last_part == "state" || last_part == "province" => Some("state"),
                _ if last_part.contains("zip") || last_part.contains("postal") => {
                    Some("postal_code")
                }
                _ if last_part == "country" => Some("country"),
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
impl ImportConnector for XmlImporter {
    fn metadata(&self) -> ConnectorMetadata {
        ConnectorMetadata {
            id: "xml".to_string(),
            name: "XML File".to_string(),
            description: "Import from generic XML files with automatic record pattern detection"
                .to_string(),
            supported_extensions: vec!["xml".to_string()],
            supported_mime_types: vec!["application/xml".to_string(), "text/xml".to_string()],
            format: ImportFormat::Xml,
            required_auth_scopes: vec![],
            version: "1.0.0".to_string(),
        }
    }

    async fn parse(&self, file_path: &Path) -> Result<ParseResult, ImportError> {
        let content = std::fs::read_to_string(file_path)?;

        // Detect the repeating record element
        let record_element = Self::detect_record_pattern(&content)?;
        let record_tag = record_element.as_bytes();

        // Parse all records
        let mut reader = Reader::from_str(&content);
        reader.trim_text(true);

        let mut rows = Vec::new();
        let mut warnings = Vec::new();
        let mut buf = Vec::new();
        let mut _in_record = false;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    if e.name().as_ref() == record_tag {
                        _in_record = true;
                        match Self::parse_record(&mut reader, record_tag, &mut buf) {
                            Ok(mut record) => {
                                if !record.is_empty() {
                                    record.insert("source".to_string(), "xml".to_string());
                                    rows.push(record);
                                }
                            }
                            Err(e) => {
                                warnings.push(format!("Failed to parse record: {}", e));
                            }
                        }
                        _in_record = false;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(ImportError::Other(format!("XML parse error: {}", e))),
                _ => {}
            }
            buf.clear();
        }

        if rows.is_empty() {
            return Err(ImportError::Other(
                "No valid records found in XML file".to_string(),
            ));
        }

        // Detect fields and generate mappings
        let detected_fields = Self::detect_fields(&rows);
        let suggested_mappings = Self::suggest_mappings(&detected_fields);

        // Build metadata
        let mut metadata = HashMap::new();
        metadata.insert("format".to_string(), "xml".to_string());
        metadata.insert("record_element".to_string(), record_element);
        metadata.insert("total_records".to_string(), rows.len().to_string());
        metadata.insert("detected_fields".to_string(), detected_fields.join(", "));

        Ok(ParseResult {
            rows,
            suggested_mappings,
            metadata,
            warnings,
        })
    }
}

impl Default for XmlImporter {
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
    async fn test_simple_contacts_xml() {
        let xml_content = r#"<?xml version="1.0"?>
<contacts>
    <contact>
        <first_name>John</first_name>
        <last_name>Doe</last_name>
        <email>john@example.com</email>
        <phone>555-1234</phone>
    </contact>
    <contact>
        <first_name>Jane</first_name>
        <last_name>Smith</last_name>
        <email>jane@example.com</email>
    </contact>
</contacts>"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", xml_content).unwrap();

        let connector = XmlImporter::new();
        let result = connector.parse(temp_file.path()).await.unwrap();

        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.rows[0].get("first_name").unwrap(), "John");
        assert_eq!(result.rows[1].get("first_name").unwrap(), "Jane");
        assert_eq!(result.metadata.get("record_element").unwrap(), "contact");
    }

    #[tokio::test]
    async fn test_xml_with_attributes() {
        let xml_content = r#"<?xml version="1.0"?>
<people>
    <person id="1" type="customer">
        <name>John Doe</name>
        <contact email="john@example.com" phone="555-1234"/>
    </person>
    <person id="2" type="vendor">
        <name>Jane Smith</name>
        <contact email="jane@example.com"/>
    </person>
</people>"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", xml_content).unwrap();

        let connector = XmlImporter::new();
        let result = connector.parse(temp_file.path()).await.unwrap();

        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.rows[0].get("name").unwrap(), "John Doe");
        // Attributes on nested elements are prefixed with element.attribute
        assert!(
            result.rows[0].get("contact@email").is_some() || result.rows[0].get("email").is_some()
        );
        // Check that we got the email value one way or another
        let has_email = result.rows[0]
            .values()
            .any(|v| v.contains("john@example.com"));
        assert!(has_email, "Expected to find john@example.com in record");
        assert_eq!(result.metadata.get("record_element").unwrap(), "person");
    }

    #[tokio::test]
    async fn test_nested_xml() {
        let xml_content = r#"<?xml version="1.0"?>
<directory>
    <entry>
        <personal>
            <firstname>John</firstname>
            <lastname>Doe</lastname>
        </personal>
        <contact>
            <email>john@example.com</email>
            <phone>555-1234</phone>
        </contact>
    </entry>
</directory>"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", xml_content).unwrap();

        let connector = XmlImporter::new();
        let result = connector.parse(temp_file.path()).await.unwrap();

        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0].get("personal.firstname").unwrap(), "John");
        assert_eq!(
            result.rows[0].get("contact.email").unwrap(),
            "john@example.com"
        );
    }
}
