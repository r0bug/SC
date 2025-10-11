use crate::config::ValidationRules;
use regex::Regex;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub stats: ValidationStats,
}

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub row: usize,
    pub field: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct ValidationWarning {
    pub row: usize,
    pub field: String,
    pub message: String,
}

#[derive(Debug, Clone, Default)]
pub struct ValidationStats {
    pub total_rows: usize,
    pub valid_rows: usize,
    pub invalid_rows: usize,
    pub duplicate_rows: usize,
    pub missing_required_fields: usize,
}

pub struct BatchValidator {
    rules: ValidationRules,
    email_regex: Regex,
    phone_regex: Regex,
    seen_records: HashSet<String>,
}

impl BatchValidator {
    pub fn new(rules: ValidationRules) -> Self {
        Self {
            rules,
            email_regex: Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap(),
            phone_regex: Regex::new(r"^[+]?[0-9]{10,15}$").unwrap(),
            seen_records: HashSet::new(),
        }
    }

    pub fn validate_batch(&mut self, rows: &[HashMap<String, String>]) -> ValidationResult {
        let mut result = ValidationResult {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            stats: ValidationStats {
                total_rows: rows.len(),
                valid_rows: 0,
                invalid_rows: 0,
                duplicate_rows: 0,
                missing_required_fields: 0,
            },
        };

        for (index, row) in rows.iter().enumerate() {
            let row_result = self.validate_row(row, index);

            if row_result.is_valid {
                result.stats.valid_rows += 1;
            } else {
                result.stats.invalid_rows += 1;
                if result.errors.len() < self.rules.max_errors {
                    result.errors.extend(row_result.errors);
                }
            }

            result.warnings.extend(row_result.warnings);

            // Check if we've hit the max error threshold
            if result.errors.len() >= self.rules.max_errors {
                result.is_valid = false;
                break;
            }
        }

        result.is_valid = result.errors.is_empty();
        result
    }

    fn validate_row(&mut self, row: &HashMap<String, String>, index: usize) -> ValidationResult {
        let mut result = ValidationResult {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            stats: ValidationStats::default(),
        };

        // Check required fields
        for field in &self.rules.required_fields {
            if !row.contains_key(field) || row[field].trim().is_empty() {
                result.errors.push(ValidationError {
                    row: index,
                    field: field.clone(),
                    message: format!("Required field '{}' is missing or empty", field),
                });
                result.is_valid = false;
            }
        }

        // Validate email format
        if self.rules.validate_emails {
            if let Some(email) = row.get("email") {
                if !email.is_empty() && !self.email_regex.is_match(email) {
                    result.errors.push(ValidationError {
                        row: index,
                        field: "email".to_string(),
                        message: format!("Invalid email format: {}", email),
                    });
                    result.is_valid = false;
                }
            }
        }

        // Validate phone format
        if self.rules.validate_phones {
            if let Some(phone) = row.get("phone") {
                if !phone.is_empty() {
                    let cleaned_phone: String = phone
                        .chars()
                        .filter(|c| c.is_ascii_digit() || *c == '+')
                        .collect();

                    if !self.phone_regex.is_match(&cleaned_phone) {
                        result.warnings.push(ValidationWarning {
                            row: index,
                            field: "phone".to_string(),
                            message: format!("Phone number may be invalid: {}", phone),
                        });
                    }
                }
            }
        }

        // Check for duplicates
        if self.rules.skip_duplicates {
            let key = self.generate_duplicate_key(row);
            if self.seen_records.contains(&key) {
                result.warnings.push(ValidationWarning {
                    row: index,
                    field: "duplicate".to_string(),
                    message: "Duplicate record detected".to_string(),
                });
            } else {
                self.seen_records.insert(key);
            }
        }

        result
    }

    fn generate_duplicate_key(&self, row: &HashMap<String, String>) -> String {
        // Create a unique key based on important fields
        let mut key_parts = Vec::new();

        if let Some(first_name) = row.get("first_name") {
            key_parts.push(first_name.to_lowercase());
        }

        if let Some(last_name) = row.get("last_name") {
            key_parts.push(last_name.to_lowercase());
        }

        if let Some(email) = row.get("email") {
            key_parts.push(email.to_lowercase());
        }

        if let Some(phone) = row.get("phone") {
            let cleaned: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
            key_parts.push(cleaned);
        }

        key_parts.join("|")
    }

    pub fn reset(&mut self) {
        self.seen_records.clear();
    }
}
