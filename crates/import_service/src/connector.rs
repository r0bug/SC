use crate::{
    config::{ImportConfig, ImportFormat},
    ImportError,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use uuid::Uuid;

/// Metadata describing an import connector's capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorMetadata {
    /// Unique identifier for the connector
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description of what this connector imports
    pub description: String,
    /// Supported file extensions (e.g., ["xml", "csv"])
    pub supported_extensions: Vec<String>,
    /// MIME types supported
    pub supported_mime_types: Vec<String>,
    /// Import format this connector handles
    pub format: ImportFormat,
    /// OAuth scopes required (for API-based imports)
    pub required_auth_scopes: Vec<String>,
    /// Version of the connector
    pub version: String,
}

/// Result from a connector's parsing operation
#[derive(Debug, Clone)]
pub struct ParseResult {
    /// Parsed rows of data
    pub rows: Vec<HashMap<String, String>>,
    /// Detected field mappings (source -> suggested target)
    pub suggested_mappings: Vec<(String, String)>,
    /// Additional metadata extracted from the file
    pub metadata: HashMap<String, String>,
    /// Warnings encountered during parsing
    pub warnings: Vec<String>,
}

/// Trait that all import connectors must implement
#[async_trait]
pub trait ImportConnector: Send + Sync {
    /// Returns metadata about this connector
    fn metadata(&self) -> ConnectorMetadata;

    /// Check if this connector can handle the given file
    fn can_handle(&self, file_path: &Path) -> bool {
        let metadata = self.metadata();

        if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
            metadata.supported_extensions.contains(&ext.to_lowercase())
        } else {
            false
        }
    }

    /// Parse the file and return structured data
    async fn parse(&self, file_path: &Path) -> Result<ParseResult, ImportError>;

    /// Validate file before parsing (optional, quick checks)
    async fn validate_file(&self, file_path: &Path) -> Result<(), ImportError> {
        // Default implementation - just check file exists and is readable
        if !file_path.exists() {
            return Err(ImportError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "File not found",
            )));
        }
        Ok(())
    }

    /// Auto-detect configuration from file content
    async fn auto_detect_config(&self, file_path: &Path) -> Result<ImportConfig, ImportError> {
        let parse_result = self.parse(file_path).await?;
        let metadata = self.metadata();

        let mut config = ImportConfig::new(
            format!("Auto-detected: {}", metadata.name),
            metadata.format.clone(),
        );

        // Add suggested mappings
        for (source, target) in parse_result.suggested_mappings {
            config.add_mapping(crate::config::ColumnMapping {
                source_column: source,
                target_field: target,
                transform: None,
                required: false,
                default_value: None,
            });
        }

        Ok(config)
    }

    /// Get sample data for preview (first N rows)
    async fn get_preview(
        &self,
        file_path: &Path,
        limit: usize,
    ) -> Result<ParseResult, ImportError> {
        let mut result = self.parse(file_path).await?;
        result.rows.truncate(limit);
        Ok(result)
    }
}

/// Registry for managing import connectors
pub struct ConnectorRegistry {
    connectors: HashMap<String, Box<dyn ImportConnector>>,
}

impl ConnectorRegistry {
    pub fn new() -> Self {
        Self {
            connectors: HashMap::new(),
        }
    }

    /// Register a new connector
    pub fn register(&mut self, connector: Box<dyn ImportConnector>) {
        let id = connector.metadata().id.clone();
        self.connectors.insert(id, connector);
    }

    /// Get a connector by ID
    pub fn get(&self, id: &str) -> Option<&Box<dyn ImportConnector>> {
        self.connectors.get(id)
    }

    /// Find the best connector for a given file
    pub fn find_connector(&self, file_path: &Path) -> Option<&Box<dyn ImportConnector>> {
        self.connectors.values().find(|c| c.can_handle(file_path))
    }

    /// List all registered connectors
    pub fn list_connectors(&self) -> Vec<ConnectorMetadata> {
        self.connectors.values().map(|c| c.metadata()).collect()
    }

    /// Get connectors by format
    pub fn get_by_format(&self, format: &ImportFormat) -> Vec<&Box<dyn ImportConnector>> {
        self.connectors
            .values()
            .filter(|c| &c.metadata().format == format)
            .collect()
    }
}

impl Default for ConnectorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockConnector;

    #[async_trait]
    impl ImportConnector for MockConnector {
        fn metadata(&self) -> ConnectorMetadata {
            ConnectorMetadata {
                id: "test".to_string(),
                name: "Test Connector".to_string(),
                description: "A test connector".to_string(),
                supported_extensions: vec!["test".to_string()],
                supported_mime_types: vec!["text/test".to_string()],
                format: ImportFormat::Csv,
                required_auth_scopes: vec![],
                version: "1.0.0".to_string(),
            }
        }

        async fn parse(&self, _file_path: &Path) -> Result<ParseResult, ImportError> {
            Ok(ParseResult {
                rows: vec![],
                suggested_mappings: vec![],
                metadata: HashMap::new(),
                warnings: vec![],
            })
        }
    }

    #[test]
    fn test_registry() {
        let mut registry = ConnectorRegistry::new();
        registry.register(Box::new(MockConnector));

        assert_eq!(registry.list_connectors().len(), 1);
        assert!(registry.get("test").is_some());
    }
}
