use crate::config::{ColumnMapping, Transform};
use crate::ImportError;
use std::collections::HashMap;

pub struct DataMapper {
    mappings: Vec<ColumnMapping>,
}

impl DataMapper {
    pub fn new(mappings: Vec<ColumnMapping>) -> Self {
        Self { mappings }
    }

    pub fn map_row(
        &self,
        source_row: &HashMap<String, String>,
    ) -> Result<HashMap<String, String>, ImportError> {
        let mut mapped_row = HashMap::new();

        for mapping in &self.mappings {
            let value = source_row
                .get(&mapping.source_column)
                .or_else(|| mapping.default_value.as_ref())
                .cloned();

            if value.is_none() && mapping.required {
                return Err(ImportError::MappingError(format!(
                    "Required field '{}' is missing",
                    mapping.source_column
                )));
            }

            if let Some(mut val) = value {
                if let Some(transform) = &mapping.transform {
                    val = self.apply_transform(val, transform)?;
                }
                mapped_row.insert(mapping.target_field.clone(), val);
            } else if let Some(default) = &mapping.default_value {
                mapped_row.insert(mapping.target_field.clone(), default.clone());
            }
        }

        Ok(mapped_row)
    }

    fn apply_transform(&self, value: String, transform: &Transform) -> Result<String, ImportError> {
        match transform {
            Transform::Lowercase => Ok(value.to_lowercase()),
            Transform::Uppercase => Ok(value.to_uppercase()),
            Transform::TrimWhitespace => Ok(value.trim().to_string()),
            Transform::DateFormat(_format) => {
                // Simple date formatting - in production would use chrono
                Ok(value)
            }
            Transform::PhoneFormat => {
                // Remove non-numeric characters except + at the beginning
                let cleaned = if value.starts_with('+') {
                    format!(
                        "+{}",
                        value[1..]
                            .chars()
                            .filter(|c| c.is_ascii_digit())
                            .collect::<String>()
                    )
                } else {
                    value.chars().filter(|c| c.is_ascii_digit()).collect()
                };
                Ok(cleaned)
            }
            Transform::EmailNormalize => Ok(value.trim().to_lowercase()),
            Transform::Custom(_name) => {
                // Custom transformation logic would go here
                // For now, just return the value
                Ok(value)
            }
        }
    }

    pub fn auto_detect_mappings(
        headers: &[String],
        target_fields: &[String],
    ) -> Vec<ColumnMapping> {
        let mut mappings = Vec::new();

        for header in headers {
            if let Some(target) = find_best_match(header, target_fields) {
                mappings.push(ColumnMapping {
                    source_column: header.clone(),
                    target_field: target.clone(),
                    transform: None,
                    required: false,
                    default_value: None,
                });
            }
        }

        mappings
    }
}

fn find_best_match(source: &str, targets: &[String]) -> Option<String> {
    let source_lower = source.to_lowercase();

    // Exact match
    for target in targets {
        if target.to_lowercase() == source_lower {
            return Some(target.clone());
        }
    }

    // Common variations
    let mappings = vec![
        ("firstname", "first_name"),
        ("lastname", "last_name"),
        ("email_address", "email"),
        ("phone_number", "phone"),
        ("mobile", "phone"),
        ("company", "organization"),
        ("job_title", "title"),
    ];

    for (variant, standard) in mappings {
        if source_lower.contains(variant) {
            for target in targets {
                if target == standard {
                    return Some(target.clone());
                }
            }
        }
    }

    // Fuzzy matching - simple implementation
    for target in targets {
        let target_lower = target.to_lowercase();
        if source_lower.contains(&target_lower) || target_lower.contains(&source_lower) {
            return Some(target.clone());
        }
    }

    None
}
