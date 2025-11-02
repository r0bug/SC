use crate::{
    config::ImportFormat,
    connector::{ConnectorMetadata, ImportConnector, ParseResult},
    ImportError,
};
use ai_middleware::SegmindClient;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tracing::{info, warn};

/// SmartImporter - AI-powered format detection and field mapping
///
/// This connector uses Segmind AI to:
/// 1. Detect file format (CSV, JSON, XML, HTML, text)
/// 2. Analyze data structure
/// 3. Suggest field mappings to contact fields
///
/// It should be registered FIRST in the connector registry to handle
/// unknown formats before falling back to specific parsers.
pub struct SmartImporter {
    ai_client: SegmindClient,
    sample_size: usize,
}

impl SmartImporter {
    pub fn new(ai_client: SegmindClient) -> Self {
        Self {
            ai_client,
            sample_size: 2000, // Read first 2000 bytes for analysis
        }
    }

    pub fn with_sample_size(mut self, size: usize) -> Self {
        self.sample_size = size;
        self
    }

    /// Read a sample of the file for AI analysis
    async fn read_sample(&self, file_path: &Path) -> Result<String, ImportError> {
        let content = std::fs::read(file_path)?;

        // Take first N bytes, but try to cut at a newline for better analysis
        let sample = if content.len() <= self.sample_size {
            String::from_utf8_lossy(&content).to_string()
        } else {
            let mut end = self.sample_size;
            // Try to find a newline near the sample size
            while end < content.len() && end < self.sample_size + 200 {
                if content[end] == b'\n' {
                    break;
                }
                end += 1;
            }
            String::from_utf8_lossy(&content[..end]).to_string()
        };

        Ok(sample)
    }

    /// Use AI to analyze the file sample and detect format + fields
    async fn analyze_with_ai(&self, sample: &str) -> Result<FormatAnalysis, ImportError> {
        let prompt = format!(
            r#"Analyze this data file sample and provide a JSON response with the following structure:

{{
  "format": "csv|json|xml|html|text|unknown",
  "confidence": 0.0-1.0,
  "detected_fields": ["field1", "field2", ...],
  "suggested_mappings": {{
    "source_field": "target_contact_field"
  }},
  "structure_notes": "brief description",
  "record_separator": "what separates records (newline, XML tags, JSON array, etc)"
}}

Target contact fields you can map to:
- first_name, last_name, full_name
- email, phone, mobile, work_phone
- organization, title, department
- address, city, state, postal_code, country
- website, linkedin_url, twitter_handle
- birthday, notes, tags

Data sample:
```
{}
```

Respond ONLY with valid JSON. Be specific about field mappings."#,
            sample
        );

        info!("[SmartImporter] Sending sample to AI for analysis");

        let response = self.ai_client.generate_suggestion(&prompt).await
            .map_err(|e| ImportError::Other(format!("AI analysis failed: {}", e)))?;

        // Try to parse the AI response as JSON
        match self.parse_ai_response(&response.text) {
            Ok(analysis) => {
                info!(
                    "[SmartImporter] AI detected format: {:?} (confidence: {})",
                    analysis.format, analysis.confidence
                );
                Ok(analysis)
            }
            Err(e) => {
                warn!("[SmartImporter] Failed to parse AI response as JSON: {}", e);
                warn!("[SmartImporter] AI response was: {}", response.text);

                // Fallback: try to extract format from text response
                Ok(self.extract_format_from_text(&response.text))
            }
        }
    }

    /// Parse structured JSON response from AI
    fn parse_ai_response(&self, text: &str) -> Result<FormatAnalysis, ImportError> {
        // Try to extract JSON from markdown code blocks if present
        let json_text = if text.contains("```json") {
            text.split("```json")
                .nth(1)
                .and_then(|s| s.split("```").next())
                .unwrap_or(text)
                .trim()
        } else if text.contains("```") {
            text.split("```")
                .nth(1)
                .and_then(|s| s.split("```").next())
                .unwrap_or(text)
                .trim()
        } else {
            text.trim()
        };

        let analysis: AiAnalysisResponse = serde_json::from_str(json_text)
            .map_err(|e| ImportError::Other(format!("JSON parse error: {}", e)))?;

        // Convert to internal format
        let format = match analysis.format.to_lowercase().as_str() {
            "csv" => ImportFormat::Csv,
            "json" => ImportFormat::Json,
            "xml" => ImportFormat::Xml,
            "html" => ImportFormat::Html,
            "text" | "txt" => ImportFormat::Text,
            _ => ImportFormat::Unknown,
        };

        let suggested_mappings: Vec<(String, String)> = analysis
            .suggested_mappings
            .into_iter()
            .collect();

        Ok(FormatAnalysis {
            format,
            confidence: analysis.confidence,
            detected_fields: analysis.detected_fields,
            suggested_mappings,
            structure_notes: analysis.structure_notes,
            record_separator: analysis.record_separator,
        })
    }

    /// Fallback: extract format from unstructured text response
    fn extract_format_from_text(&self, text: &str) -> FormatAnalysis {
        let lower = text.to_lowercase();

        let format = if lower.contains("csv") || lower.contains("comma-separated") {
            ImportFormat::Csv
        } else if lower.contains("json") {
            ImportFormat::Json
        } else if lower.contains("xml") {
            ImportFormat::Xml
        } else if lower.contains("html") || lower.contains("<table") {
            ImportFormat::Html
        } else if lower.contains("text") || lower.contains("plain") {
            ImportFormat::Text
        } else {
            ImportFormat::Unknown
        };

        // Try to extract field names mentioned in the response
        let detected_fields = vec![];

        FormatAnalysis {
            format,
            confidence: 0.5, // Lower confidence for text extraction
            detected_fields,
            suggested_mappings: vec![],
            structure_notes: text.to_string(),
            record_separator: "unknown".to_string(),
        }
    }
}

#[async_trait]
impl ImportConnector for SmartImporter {
    fn metadata(&self) -> ConnectorMetadata {
        ConnectorMetadata {
            id: "smart_importer".to_string(),
            name: "Smart AI Importer".to_string(),
            description: "AI-powered format detection and field mapping using Segmind".to_string(),
            supported_extensions: vec![
                "csv".to_string(),
                "json".to_string(),
                "xml".to_string(),
                "html".to_string(),
                "htm".to_string(),
                "txt".to_string(),
                "text".to_string(),
            ],
            supported_mime_types: vec![
                "text/csv".to_string(),
                "application/json".to_string(),
                "text/xml".to_string(),
                "application/xml".to_string(),
                "text/html".to_string(),
                "text/plain".to_string(),
            ],
            format: ImportFormat::Unknown, // Will be detected by AI
            required_auth_scopes: vec![],
            version: "1.0.0".to_string(),
        }
    }

    fn can_handle(&self, file_path: &Path) -> bool {
        // SmartImporter can attempt to handle any text-based file
        // Check if file exists and is readable
        if !file_path.exists() {
            return false;
        }

        // Try to read a few bytes to check if it's text-based
        if let Ok(sample) = std::fs::read(file_path) {
            if sample.len() > 0 {
                // Check if mostly text (heuristic: >80% printable ASCII/UTF-8)
                let text_bytes = sample.iter()
                    .take(1000)
                    .filter(|&&b| b >= 32 && b <= 126 || b == b'\n' || b == b'\r' || b == b'\t' || b > 127)
                    .count();

                return text_bytes as f32 / sample.len().min(1000) as f32 > 0.8;
            }
        }

        false
    }

    async fn parse(&self, file_path: &Path) -> Result<ParseResult, ImportError> {
        info!("[SmartImporter] Starting AI-powered import for: {:?}", file_path);

        // Step 1: Read sample
        let sample = self.read_sample(file_path).await?;

        if sample.is_empty() {
            return Err(ImportError::Other("File is empty".to_string()));
        }

        // Step 2: Analyze with AI
        let analysis = self.analyze_with_ai(&sample).await?;

        // Step 3: Create metadata from analysis
        let mut metadata = HashMap::new();
        metadata.insert("detected_format".to_string(), format!("{:?}", analysis.format));
        metadata.insert("confidence".to_string(), analysis.confidence.to_string());
        metadata.insert("structure_notes".to_string(), analysis.structure_notes.clone());
        metadata.insert("record_separator".to_string(), analysis.record_separator.clone());
        metadata.insert("ai_analyzed".to_string(), "true".to_string());

        // Step 4: For preview, parse a few sample records
        // This is a simplified version - actual parsing would be done by format-specific parsers
        let rows = vec![]; // Will be populated by specific parsers after format detection

        let warnings = if analysis.confidence < 0.7 {
            vec![format!(
                "Low confidence format detection ({:.1}%). Please verify the suggested mappings.",
                analysis.confidence * 100.0
            )]
        } else {
            vec![]
        };

        info!(
            "[SmartImporter] Analysis complete. Format: {:?}, Fields: {:?}, Mappings: {}",
            analysis.format,
            analysis.detected_fields,
            analysis.suggested_mappings.len()
        );

        Ok(ParseResult {
            rows,
            suggested_mappings: analysis.suggested_mappings,
            metadata,
            warnings,
        })
    }
}

/// Internal structure for AI analysis results
#[derive(Debug, Clone)]
struct FormatAnalysis {
    format: ImportFormat,
    confidence: f32,
    detected_fields: Vec<String>,
    suggested_mappings: Vec<(String, String)>,
    structure_notes: String,
    record_separator: String,
}

/// JSON structure expected from AI response
#[derive(Debug, Deserialize, Serialize)]
struct AiAnalysisResponse {
    format: String,
    confidence: f32,
    detected_fields: Vec<String>,
    suggested_mappings: HashMap<String, String>,
    structure_notes: String,
    record_separator: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_smart_importer_csv_sample() {
        let csv_content = "First Name,Last Name,Email,Phone\nJohn,Doe,john@example.com,555-1234\nJane,Smith,jane@example.com,555-5678";

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_content).unwrap();

        // Create with mock Segmind client
        let ai_client = SegmindClient::new(None); // Mock mode
        let importer = SmartImporter::new(ai_client);

        let sample = importer.read_sample(temp_file.path()).await.unwrap();
        assert!(sample.contains("First Name"));
        assert!(sample.contains("Email"));
    }

    #[test]
    fn test_can_handle_text_files() {
        let ai_client = SegmindClient::new(None);
        let importer = SmartImporter::new(ai_client);

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "Some text content").unwrap();

        assert!(importer.can_handle(temp_file.path()));
    }
}
