use crate::{
    config::{ConfigRepository, ImportConfig, ImportFormat},
    mapper::DataMapper,
    transaction::{ImportSummary, ImportTransaction, Operation},
    validator::{BatchValidator, ValidationResult},
    ImportError,
};
use core_domain::Contact;
use local_store::ContactRepository;
use sqlx::{Pool, Sqlite};
use std::collections::HashMap;
use std::path::Path;
use tracing::warn;
use uuid::Uuid;

pub struct ImportService {
    pool: Pool<Sqlite>,
    config_repo: ConfigRepository,
}

#[derive(Debug, Clone)]
pub struct ImportResult {
    pub success: bool,
    pub summary: Option<ImportSummary>,
    pub validation: ValidationResult,
    pub config_id: Uuid,
}

impl ImportService {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self {
            pool,
            config_repo: ConfigRepository::new(),
        }
    }

    pub async fn import_file(
        &self,
        file_path: &Path,
        config_id: Option<Uuid>,
        dry_run: bool,
        user_id: Uuid,
    ) -> Result<ImportResult, ImportError> {
        // Load or create config
        let config = if let Some(id) = config_id {
            self.config_repo.load(&id)?
        } else {
            // Auto-detect format and create default config
            let format = detect_format(file_path)?;
            let mut config = ImportConfig::new("Auto-generated".to_string(), format);

            // Auto-detect mappings based on file headers
            if let ImportFormat::Csv = config.format {
                let headers = read_csv_headers(file_path)?;
                let target_fields = get_contact_field_names();
                config.mappings = DataMapper::auto_detect_mappings(&headers, &target_fields);
            }

            // Save the auto-generated config
            self.config_repo.save(&config)?;
            config
        };

        // Parse the file based on format
        let rows = match config.format {
            ImportFormat::Csv => parse_csv(file_path)?,
            ImportFormat::Vcard => parse_vcard(file_path)?,
            ImportFormat::Json => parse_json(file_path)?,
            ImportFormat::Sms => parse_sms(file_path)?,
            _ => return Err(ImportError::Other("Unsupported format".to_string())),
        };

        // Validate the data
        let mut validator = BatchValidator::new(config.validation_rules.clone());
        let validation = validator.validate_batch(&rows);

        if !validation.is_valid && !dry_run {
            return Ok(ImportResult {
                success: false,
                summary: None,
                validation,
                config_id: config.id,
            });
        }

        // If dry run, return validation results only
        if dry_run {
            return Ok(ImportResult {
                success: validation.is_valid,
                summary: None,
                validation,
                config_id: config.id,
            });
        }

        // Map and import the data
        let mapper = DataMapper::new(config.mappings.clone());
        let mut transaction = ImportTransaction::new(&self.pool).await?;

        let contact_repo = ContactRepository::new(&self.pool);

        for (index, row) in rows.iter().enumerate() {
            // Skip invalid rows
            let is_valid = validation.errors.iter().all(|e| e.row != index);

            if !is_valid {
                transaction.log_change("contact".to_string(), Uuid::nil(), Operation::Skip);
                continue;
            }

            // Map the row to contact fields
            let mapped = mapper.map_row(row)?;

            // Create contact from mapped data
            let contact = create_contact_from_mapped(&mapped, user_id)?;

            // Check if contact exists (for update vs insert)
            let existing = contact_repo
                .search(contact.email.as_deref().unwrap_or(""), user_id)
                .await
                .ok()
                .and_then(|results| results.into_iter().next());

            if let Some(existing_contact) = existing {
                // Update existing contact
                // In a real implementation, we'd merge the data
                transaction.log_change(
                    "contact".to_string(),
                    existing_contact.id,
                    Operation::Update,
                );
            } else {
                // Insert new contact
                contact_repo
                    .create(&contact)
                    .await
                    .map_err(|e| ImportError::Other(e.to_string()))?;

                transaction.add_imported_id(contact.id);
                transaction.log_change("contact".to_string(), contact.id, Operation::Insert);
            }

            // Checkpoint every N records
            if index % 100 == 0 {
                transaction.checkpoint().await?;
            }
        }

        // Commit the transaction
        let summary = transaction.commit().await?;

        Ok(ImportResult {
            success: true,
            summary: Some(summary),
            validation,
            config_id: config.id,
        })
    }

    pub async fn rollback_import(&self, transaction_id: Uuid) -> Result<(), ImportError> {
        // In a real implementation, we would:
        // 1. Load the transaction log
        // 2. Delete all imported records
        // 3. Restore any updated records to their previous state

        warn!(
            "Rollback not fully implemented for transaction: {}",
            transaction_id
        );
        Ok(())
    }

    pub fn list_configs(&self) -> Result<Vec<ImportConfig>, ImportError> {
        self.config_repo.list().map_err(ImportError::IoError)
    }

    pub fn save_config(&self, config: &ImportConfig) -> Result<(), ImportError> {
        self.config_repo.save(config).map_err(ImportError::IoError)
    }
}

fn detect_format(path: &Path) -> Result<ImportFormat, ImportError> {
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .ok_or_else(|| ImportError::Other("No file extension".to_string()))?;

    match extension.to_lowercase().as_str() {
        "csv" => Ok(ImportFormat::Csv),
        "vcf" | "vcard" => Ok(ImportFormat::Vcard),
        "json" => Ok(ImportFormat::Json),
        "txt" if path.to_str().is_some_and(|s| s.contains("sms")) => Ok(ImportFormat::Sms),
        _ => Err(ImportError::Other(format!("Unknown format: {}", extension))),
    }
}

fn read_csv_headers(path: &Path) -> Result<Vec<String>, ImportError> {
    let mut reader = csv::Reader::from_path(path)?;
    let headers = reader.headers()?.iter().map(|h| h.to_string()).collect();
    Ok(headers)
}

fn parse_csv(path: &Path) -> Result<Vec<HashMap<String, String>>, ImportError> {
    let mut reader = csv::Reader::from_path(path)?;
    let headers: Vec<String> = reader.headers()?.iter().map(|h| h.to_string()).collect();

    let mut rows = Vec::new();
    for result in reader.records() {
        let record = result?;
        let mut row = HashMap::new();

        for (index, field) in record.iter().enumerate() {
            if let Some(header) = headers.get(index) {
                row.insert(header.clone(), field.to_string());
            }
        }

        rows.push(row);
    }

    Ok(rows)
}

fn parse_vcard(path: &Path) -> Result<Vec<HashMap<String, String>>, ImportError> {
    // Simplified vCard parsing - in production would use vcard crate properly
    let content = std::fs::read_to_string(path)?;
    let mut rows = Vec::new();
    let mut current = HashMap::new();

    for line in content.lines() {
        if line.starts_with("BEGIN:VCARD") {
            current = HashMap::new();
        } else if line.starts_with("END:VCARD") {
            if !current.is_empty() {
                rows.push(current.clone());
            }
        } else if let Some((key, value)) = line.split_once(':') {
            match key {
                "FN" => {
                    let parts: Vec<&str> = value.split_whitespace().collect();
                    if let Some(first) = parts.first() {
                        current.insert("first_name".to_string(), first.to_string());
                    }
                    if let Some(last) = parts.last() {
                        if parts.len() > 1 {
                            current.insert("last_name".to_string(), last.to_string());
                        }
                    }
                }
                "EMAIL" => {
                    current.insert("email".to_string(), value.to_string());
                }
                "TEL" => {
                    current.insert("phone".to_string(), value.to_string());
                }
                "ORG" => {
                    current.insert("organization".to_string(), value.to_string());
                }
                "TITLE" => {
                    current.insert("title".to_string(), value.to_string());
                }
                _ => {}
            }
        }
    }

    Ok(rows)
}

fn parse_json(path: &Path) -> Result<Vec<HashMap<String, String>>, ImportError> {
    let content = std::fs::read_to_string(path)?;
    let json: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| ImportError::Other(e.to_string()))?;

    let mut rows = Vec::new();

    if let Some(array) = json.as_array() {
        for item in array {
            let mut row = HashMap::new();
            if let Some(obj) = item.as_object() {
                for (key, value) in obj {
                    let str_value = match value {
                        serde_json::Value::String(s) => s.clone(),
                        serde_json::Value::Number(n) => n.to_string(),
                        serde_json::Value::Bool(b) => b.to_string(),
                        _ => continue,
                    };
                    row.insert(key.clone(), str_value);
                }
            }
            rows.push(row);
        }
    }

    Ok(rows)
}

fn parse_sms(path: &Path) -> Result<Vec<HashMap<String, String>>, ImportError> {
    // Simple SMS backup parsing - would need proper implementation
    let content = std::fs::read_to_string(path)?;
    let mut rows = Vec::new();

    for line in content.lines() {
        if line.contains("From:") || line.contains("To:") {
            let mut row = HashMap::new();

            // Extract phone number
            if let Some(phone) = line.split(':').nth(1) {
                row.insert("phone".to_string(), phone.trim().to_string());
                row.insert(
                    "first_name".to_string(),
                    format!("SMS Contact {}", phone.trim()),
                );
                rows.push(row);
            }
        }
    }

    Ok(rows)
}

fn get_contact_field_names() -> Vec<String> {
    vec![
        "first_name".to_string(),
        "last_name".to_string(),
        "email".to_string(),
        "phone".to_string(),
        "organization".to_string(),
        "title".to_string(),
        "notes".to_string(),
    ]
}

fn create_contact_from_mapped(
    data: &HashMap<String, String>,
    user_id: Uuid,
) -> Result<Contact, ImportError> {
    Ok(Contact {
        id: Uuid::new_v4(),
        first_name: data
            .get("first_name")
            .cloned()
            .unwrap_or_else(|| "Unknown".to_string()),
        last_name: data.get("last_name").cloned(),
        email: data.get("email").cloned(),
        phone: data.get("phone").cloned(),
        organization: data.get("organization").cloned(),
        title: data.get("title").cloned(),
        notes: data.get("notes").cloned(),
        social_handles: vec![],
        tags: vec![],
        projects: vec![],
        groups: vec![],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        created_by: user_id,
        version: 1,
        last_synced_at: None,
        metadata: serde_json::json!({}),
    })
}
