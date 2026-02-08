use crate::ImportError;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DuplicateStrategy {
    /// Skip duplicate entries
    Skip,
    /// Update existing entries with new data
    Update,
    /// Merge fields from both entries
    Merge,
    /// Keep both as separate entries
    KeepBoth,
    /// Prompt user for decision
    Ask,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MatchCriteria {
    /// Match by email address (exact)
    Email,
    /// Match by phone number (normalized)
    Phone,
    /// Match by full name (fuzzy)
    FullName,
    /// Match by email or phone
    EmailOrPhone,
    /// Custom matching logic
    Custom(Vec<String>), // Field names to match on
}

#[derive(Debug, Clone)]
pub struct DeduplicationConfig {
    pub strategy: DuplicateStrategy,
    pub match_criteria: MatchCriteria,
    pub fuzzy_threshold: f32, // 0.0 to 1.0, for fuzzy matching
    pub normalize_phone: bool,
    pub normalize_email: bool,
}

impl Default for DeduplicationConfig {
    fn default() -> Self {
        Self {
            strategy: DuplicateStrategy::Skip,
            match_criteria: MatchCriteria::EmailOrPhone,
            fuzzy_threshold: 0.85,
            normalize_phone: true,
            normalize_email: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DuplicateMatch {
    pub original_index: usize,
    pub duplicate_index: usize,
    pub match_score: f32,
    pub matched_on: String,
}

pub struct DeduplicationEngine {
    config: DeduplicationConfig,
}

impl DeduplicationEngine {
    pub fn new(config: DeduplicationConfig) -> Self {
        Self { config }
    }

    /// Find duplicates in a batch of rows
    pub fn find_duplicates(
        &self,
        rows: &[HashMap<String, String>],
    ) -> Result<Vec<DuplicateMatch>, ImportError> {
        let mut duplicates = Vec::new();
        let mut seen: HashMap<String, usize> = HashMap::new();

        for (index, row) in rows.iter().enumerate() {
            if let Some(key) = self.generate_match_key(row) {
                if let Some(&original_index) = seen.get(&key) {
                    duplicates.push(DuplicateMatch {
                        original_index,
                        duplicate_index: index,
                        match_score: 1.0, // Exact match
                        matched_on: key.clone(),
                    });
                } else {
                    seen.insert(key, index);
                }
            }
        }

        // Fuzzy matching for name-based criteria
        if matches!(self.config.match_criteria, MatchCriteria::FullName) {
            duplicates.extend(self.find_fuzzy_name_duplicates(rows)?);
        }

        Ok(duplicates)
    }

    /// Apply deduplication strategy to rows
    pub fn deduplicate(
        &self,
        mut rows: Vec<HashMap<String, String>>,
    ) -> Result<Vec<HashMap<String, String>>, ImportError> {
        let duplicates = self.find_duplicates(&rows)?;

        if duplicates.is_empty() {
            return Ok(rows);
        }

        // Track indices to remove
        let mut to_remove: HashSet<usize> = HashSet::new();

        for dup in duplicates {
            match self.config.strategy {
                DuplicateStrategy::Skip => {
                    to_remove.insert(dup.duplicate_index);
                }
                DuplicateStrategy::Update => {
                    // Update original with duplicate data
                    let duplicate_data = rows.get(dup.duplicate_index).cloned();
                    if let Some(duplicate) = duplicate_data {
                        if let Some(original) = rows.get_mut(dup.original_index) {
                            for (key, value) in duplicate {
                                if !value.is_empty() {
                                    original.insert(key.clone(), value.clone());
                                }
                            }
                        }
                    }
                    to_remove.insert(dup.duplicate_index);
                }
                DuplicateStrategy::Merge => {
                    // Merge fields intelligently
                    let duplicate_data = rows.get(dup.duplicate_index).cloned();
                    if let Some(duplicate) = duplicate_data {
                        if let Some(original) = rows.get_mut(dup.original_index) {
                            self.merge_rows(original, &duplicate);
                        }
                    }
                    to_remove.insert(dup.duplicate_index);
                }
                DuplicateStrategy::KeepBoth => {
                    // Do nothing, keep both entries
                }
                DuplicateStrategy::Ask => {
                    // Mark for user review (handled by caller)
                    if let Some(row) = rows.get_mut(dup.duplicate_index) {
                        row.insert("_duplicate_of".to_string(), dup.original_index.to_string());
                        row.insert("_needs_review".to_string(), "true".to_string());
                    }
                }
            }
        }

        // Remove marked rows (in reverse order to maintain indices)
        let mut to_remove: Vec<usize> = to_remove.into_iter().collect();
        to_remove.sort_by(|a, b| b.cmp(a));
        for index in to_remove {
            rows.remove(index);
        }

        Ok(rows)
    }

    /// Generate a match key based on configured criteria
    fn generate_match_key(&self, row: &HashMap<String, String>) -> Option<String> {
        match &self.config.match_criteria {
            MatchCriteria::Email => row.get("email").map(|e| self.normalize_email_if_needed(e)),
            MatchCriteria::Phone => row.get("phone").map(|p| self.normalize_phone_if_needed(p)),
            MatchCriteria::FullName => {
                let first = row.get("first_name").map(|s| s.as_str()).unwrap_or("");
                let last = row.get("last_name").map(|s| s.as_str()).unwrap_or("");
                if !first.is_empty() || !last.is_empty() {
                    Some(
                        format!("{} {}", first, last)
                            .to_lowercase()
                            .trim()
                            .to_string(),
                    )
                } else {
                    None
                }
            }
            MatchCriteria::EmailOrPhone => {
                if let Some(email) = row.get("email") {
                    Some(format!("email:{}", self.normalize_email_if_needed(email)))
                } else {
                    row.get("phone")
                        .map(|phone| format!("phone:{}", self.normalize_phone_if_needed(phone)))
                }
            }
            MatchCriteria::Custom(fields) => {
                let key_parts: Vec<String> = fields
                    .iter()
                    .filter_map(|field| row.get(field).map(|v| v.to_lowercase()))
                    .collect();

                if key_parts.is_empty() {
                    None
                } else {
                    Some(key_parts.join("|"))
                }
            }
        }
    }

    /// Find fuzzy duplicates based on name similarity
    fn find_fuzzy_name_duplicates(
        &self,
        rows: &[HashMap<String, String>],
    ) -> Result<Vec<DuplicateMatch>, ImportError> {
        let mut fuzzy_matches = Vec::new();

        for i in 0..rows.len() {
            for j in (i + 1)..rows.len() {
                let name1 = self.get_full_name(&rows[i]);
                let name2 = self.get_full_name(&rows[j]);

                if !name1.is_empty() && !name2.is_empty() {
                    let score = self.calculate_similarity(&name1, &name2);
                    if score >= self.config.fuzzy_threshold {
                        fuzzy_matches.push(DuplicateMatch {
                            original_index: i,
                            duplicate_index: j,
                            match_score: score,
                            matched_on: format!("name_fuzzy:{}", name1),
                        });
                    }
                }
            }
        }

        Ok(fuzzy_matches)
    }

    /// Get full name from row
    fn get_full_name(&self, row: &HashMap<String, String>) -> String {
        let first = row.get("first_name").map(|s| s.as_str()).unwrap_or("");
        let last = row.get("last_name").map(|s| s.as_str()).unwrap_or("");
        format!("{} {}", first, last).trim().to_lowercase()
    }

    /// Calculate string similarity (Jaro-Winkler-like)
    fn calculate_similarity(&self, s1: &str, s2: &str) -> f32 {
        if s1 == s2 {
            return 1.0;
        }

        let len1 = s1.len();
        let len2 = s2.len();

        if len1 == 0 || len2 == 0 {
            return 0.0;
        }

        // Simple Levenshtein-based similarity
        let distance = levenshtein_distance(s1, s2);
        let max_len = len1.max(len2);

        1.0 - (distance as f32 / max_len as f32)
    }

    /// Merge two rows intelligently
    fn merge_rows(
        &self,
        original: &mut HashMap<String, String>,
        duplicate: &HashMap<String, String>,
    ) {
        for (key, value) in duplicate {
            match original.get(key) {
                None => {
                    // Field doesn't exist in original, add it
                    original.insert(key.clone(), value.clone());
                }
                Some(existing_value) if existing_value.is_empty() => {
                    // Field exists but is empty, use duplicate value
                    original.insert(key.clone(), value.clone());
                }
                Some(existing_value) if key == "notes" => {
                    // Append notes
                    let merged = format!("{}\n---\n{}", existing_value, value);
                    original.insert(key.clone(), merged);
                }
                Some(existing_value) if existing_value != value => {
                    // Values differ, create alternate field
                    original.insert(format!("{}_alt", key), value.clone());
                }
                _ => {
                    // Keep original value
                }
            }
        }
    }

    /// Normalize email if configured
    fn normalize_email_if_needed(&self, email: &str) -> String {
        if self.config.normalize_email {
            normalize_email(email)
        } else {
            email.to_string()
        }
    }

    /// Normalize phone if configured
    fn normalize_phone_if_needed(&self, phone: &str) -> String {
        if self.config.normalize_phone {
            normalize_phone(phone)
        } else {
            phone.to_string()
        }
    }
}

/// Normalize email address (lowercase, trim)
fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

/// Normalize phone number (remove non-digits)
fn normalize_phone(phone: &str) -> String {
    phone.chars().filter(|c| c.is_ascii_digit()).collect()
}

/// Calculate Levenshtein distance between two strings
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.len();
    let len2 = s2.len();

    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    let mut matrix: Vec<Vec<usize>> = vec![vec![0; len2 + 1]; len1 + 1];

    for (i, row) in matrix.iter_mut().enumerate().take(len1 + 1) {
        row[0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    for (i, c1) in s1.chars().enumerate() {
        for (j, c2) in s2.chars().enumerate() {
            let cost = if c1 == c2 { 0 } else { 1 };
            matrix[i + 1][j + 1] = (matrix[i][j + 1] + 1)
                .min(matrix[i + 1][j] + 1)
                .min(matrix[i][j] + cost);
        }
    }

    matrix[len1][len2]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_email_dedup() {
        let config = DeduplicationConfig {
            strategy: DuplicateStrategy::Skip,
            match_criteria: MatchCriteria::Email,
            ..Default::default()
        };

        let engine = DeduplicationEngine::new(config);

        let rows = vec![
            HashMap::from([
                ("first_name".to_string(), "John".to_string()),
                ("email".to_string(), "john@example.com".to_string()),
            ]),
            HashMap::from([
                ("first_name".to_string(), "Johnny".to_string()),
                ("email".to_string(), "john@example.com".to_string()),
            ]),
        ];

        let duplicates = engine.find_duplicates(&rows).unwrap();
        assert_eq!(duplicates.len(), 1);
        assert_eq!(duplicates[0].original_index, 0);
        assert_eq!(duplicates[0].duplicate_index, 1);
    }

    #[test]
    fn test_phone_normalization() {
        assert_eq!(normalize_phone("+1 (555) 123-4567"), "15551234567");
        assert_eq!(normalize_phone("555-1234"), "5551234");
    }

    #[test]
    fn test_levenshtein() {
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("test", "test"), 0);
    }

    #[test]
    fn test_merge_strategy() {
        let config = DeduplicationConfig {
            strategy: DuplicateStrategy::Merge,
            match_criteria: MatchCriteria::Email,
            ..Default::default()
        };

        let engine = DeduplicationEngine::new(config);

        let rows = vec![
            HashMap::from([
                ("email".to_string(), "john@example.com".to_string()),
                ("first_name".to_string(), "John".to_string()),
                ("phone".to_string(), "".to_string()),
            ]),
            HashMap::from([
                ("email".to_string(), "john@example.com".to_string()),
                ("phone".to_string(), "555-1234".to_string()),
            ]),
        ];

        let result = engine.deduplicate(rows).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].get("phone").unwrap(), "555-1234");
    }
}
