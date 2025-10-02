use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportConfig {
    pub id: Uuid,
    pub name: String,
    pub format: ImportFormat,
    pub mappings: Vec<ColumnMapping>,
    pub validation_rules: ValidationRules,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImportFormat {
    Csv,
    Vcard,
    Json,
    Excel,
    Sms,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnMapping {
    pub source_column: String,
    pub target_field: String,
    pub transform: Option<Transform>,
    pub required: bool,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Transform {
    Lowercase,
    Uppercase,
    TrimWhitespace,
    DateFormat(String),
    PhoneFormat,
    EmailNormalize,
    Custom(String), // Custom transformation function name
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRules {
    pub skip_duplicates: bool,
    pub validate_emails: bool,
    pub validate_phones: bool,
    pub required_fields: Vec<String>,
    pub max_errors: usize,
}

impl Default for ValidationRules {
    fn default() -> Self {
        Self {
            skip_duplicates: true,
            validate_emails: true,
            validate_phones: true,
            required_fields: vec!["first_name".to_string()],
            max_errors: 100,
        }
    }
}

impl ImportConfig {
    pub fn new(name: String, format: ImportFormat) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            format,
            mappings: Vec::new(),
            validation_rules: ValidationRules::default(),
            created_at: chrono::Utc::now(),
            last_used_at: None,
        }
    }

    pub fn add_mapping(&mut self, mapping: ColumnMapping) {
        self.mappings.push(mapping);
    }

    pub fn save_to_file(&self, path: &PathBuf) -> Result<(), std::io::Error> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn load_from_file(path: &PathBuf) -> Result<Self, std::io::Error> {
        let json = std::fs::read_to_string(path)?;
        let config: Self = serde_json::from_str(&json)?;
        Ok(config)
    }

    pub fn get_mapping_for_field(&self, field_name: &str) -> Option<&ColumnMapping> {
        self.mappings
            .iter()
            .find(|m| m.target_field == field_name)
    }
}

pub struct ConfigRepository {
    config_dir: PathBuf,
}

impl ConfigRepository {
    pub fn new() -> Self {
        let mut config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        config_dir.push("sagenscontact");
        config_dir.push("import_configs");
        std::fs::create_dir_all(&config_dir).ok();

        Self { config_dir }
    }

    pub fn save(&self, config: &ImportConfig) -> Result<(), std::io::Error> {
        let file_name = format!("{}.json", config.id);
        let path = self.config_dir.join(file_name);
        config.save_to_file(&path)
    }

    pub fn load(&self, id: &Uuid) -> Result<ImportConfig, std::io::Error> {
        let file_name = format!("{}.json", id);
        let path = self.config_dir.join(file_name);
        ImportConfig::load_from_file(&path)
    }

    pub fn list(&self) -> Result<Vec<ImportConfig>, std::io::Error> {
        let mut configs = Vec::new();

        for entry in std::fs::read_dir(&self.config_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(config) = ImportConfig::load_from_file(&path) {
                    configs.push(config);
                }
            }
        }

        Ok(configs)
    }

    pub fn delete(&self, id: &Uuid) -> Result<(), std::io::Error> {
        let file_name = format!("{}.json", id);
        let path = self.config_dir.join(file_name);
        std::fs::remove_file(path)
    }
}