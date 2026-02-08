use core_domain::{
    CommunicationConcept, ConceptMatcher, DetectionMethod, MatchMode, MatcherElementType,
    SuggestionStatus,
};
use local_store::repositories::{
    CommunicationConceptRepository, ConceptMatcherRepository, ConceptRepository,
    EmailHistoryRepository, SmsHistoryRepository,
};
use local_store::DbPool;
use uuid::Uuid;

/// The concept detection engine scans communication text for concept keywords
/// and creates `CommunicationConcept` suggestions that users can confirm or deny.
pub struct ConceptDetector<'a> {
    pool: &'a DbPool,
}

/// A detected concept match from scanning text
#[derive(Debug)]
pub struct Detection {
    pub concept_id: Uuid,
    pub _keyword: String,
    pub confidence: f32,
}

/// Abstraction of communication fields for rule-based matching
#[derive(Debug, Clone)]
pub struct CommunicationFields {
    pub id: Uuid,
    pub communication_type: String,
    pub sender: String,
    pub receivers: Vec<String>,
    pub subject: Option<String>,
    pub body: Option<String>,
    pub attachment_names: Vec<String>,
}

impl CommunicationFields {
    pub fn from_email(email: &local_store::repositories::EmailMessage) -> Self {
        Self {
            id: email.id,
            communication_type: "email".to_string(),
            sender: email.from_address.clone(),
            receivers: {
                let mut r = vec![email.to_address.clone()];
                if let Some(cc) = &email.cc {
                    r.extend(cc.split(',').map(|s| s.trim().to_string()));
                }
                r
            },
            subject: Some(email.subject.clone()),
            body: email.body_text.clone(),
            attachment_names: Vec::new(), // attachments not tracked in email_history
        }
    }

    pub fn from_sms(sms: &local_store::repositories::SmsMessage) -> Self {
        Self {
            id: sms.id,
            communication_type: "sms".to_string(),
            sender: sms.phone_number.clone(),
            receivers: vec![sms.phone_number.clone()],
            subject: sms.subject.clone(),
            body: Some(sms.body.clone()),
            attachment_names: Vec::new(),
        }
    }
}

impl<'a> ConceptDetector<'a> {
    pub fn new(pool: &'a DbPool) -> Self {
        Self { pool }
    }

    /// Scan text content for concept keywords and return detected concepts.
    ///
    /// Uses case-insensitive word-boundary matching against all concept_keywords.
    /// Returns a list of detections with the matched keyword and confidence score.
    pub async fn detect_concepts(&self, text: &str) -> Result<Vec<Detection>, String> {
        if text.trim().is_empty() {
            return Ok(Vec::new());
        }

        let concept_repo = ConceptRepository::new(self.pool);

        // Get all concepts with their keywords
        let concepts = concept_repo
            .list(10000, 0)
            .await
            .map_err(|e| format!("Failed to load concepts: {}", e))?;

        let text_lower = text.to_lowercase();
        let mut detections = Vec::new();
        let mut seen_concepts = std::collections::HashSet::new();

        for concept in &concepts {
            for keyword in &concept.keywords {
                if seen_concepts.contains(&concept.id) {
                    break;
                }

                if keyword_matches(&text_lower, &keyword.keyword) {
                    seen_concepts.insert(concept.id);
                    detections.push(Detection {
                        concept_id: concept.id,
                        _keyword: keyword.keyword.clone(),
                        confidence: calculate_confidence(&text_lower, &keyword.keyword),
                    });
                }
            }
        }

        Ok(detections)
    }

    /// Scan communication text and store detections as suggestions.
    ///
    /// Creates `CommunicationConcept` records with status "suggested" for each detected concept.
    /// Skips concepts that already have a suggestion for this communication (avoids duplicates).
    pub async fn scan_and_store(
        &self,
        communication_type: &str,
        communication_id: Uuid,
        text: &str,
    ) -> Result<Vec<CommunicationConcept>, String> {
        let detections = self.detect_concepts(text).await?;
        if detections.is_empty() {
            return Ok(Vec::new());
        }

        let cc_repo = CommunicationConceptRepository::new(self.pool);

        // Get existing suggestions to avoid duplicates
        let existing = cc_repo
            .list_by_communication(communication_type, communication_id)
            .await
            .unwrap_or_default();

        let existing_concept_ids: std::collections::HashSet<Uuid> =
            existing.iter().map(|c| c.concept_id).collect();

        let now = chrono::Utc::now();
        let mut created = Vec::new();

        for detection in detections {
            if existing_concept_ids.contains(&detection.concept_id) {
                continue;
            }

            let cc = CommunicationConcept {
                id: Uuid::new_v4(),
                communication_type: communication_type.to_string(),
                communication_id,
                concept_id: detection.concept_id,
                detection_method: DetectionMethod::Keyword,
                status: SuggestionStatus::Suggested,
                confidence: Some(detection.confidence),
                reviewed_at: None,
                reviewed_by: None,
                created_at: now,
            };

            if cc_repo.create(&cc).await.is_ok() {
                created.push(cc);
            }
        }

        Ok(created)
    }

    /// Evaluate all concept matchers against a single communication using rule-based detection.
    pub async fn detect_by_rules(
        &self,
        comm: &CommunicationFields,
    ) -> Result<Vec<Detection>, String> {
        let matcher_repo = ConceptMatcherRepository::new(self.pool);
        let concepts_with_matchers = matcher_repo
            .load_all_concepts_with_matchers()
            .await
            .map_err(|e| format!("Failed to load concept matchers: {}", e))?;

        let mut detections = Vec::new();

        for cwm in &concepts_with_matchers {
            if cwm.groups.is_empty() {
                continue;
            }

            // DNF: any group passing = concept matches (OR between groups)
            let mut concept_matched = false;
            for group in &cwm.groups {
                if group.matchers.is_empty() {
                    continue;
                }

                // All matchers in group must pass (AND within group)
                let group_passed = group
                    .matchers
                    .iter()
                    .all(|matcher| evaluate_matcher(matcher, comm));

                if group_passed {
                    concept_matched = true;
                    break;
                }
            }

            if concept_matched {
                detections.push(Detection {
                    concept_id: cwm.concept_id,
                    _keyword: format!("rule:{}", cwm.concept_name),
                    confidence: 0.9,
                });
            }
        }

        Ok(detections)
    }

    /// Scan and store rule-based detections for a communication
    pub async fn scan_and_store_rules(
        &self,
        comm: &CommunicationFields,
    ) -> Result<Vec<CommunicationConcept>, String> {
        let detections = self.detect_by_rules(comm).await?;
        if detections.is_empty() {
            return Ok(Vec::new());
        }

        let cc_repo = CommunicationConceptRepository::new(self.pool);

        let existing = cc_repo
            .list_by_communication(&comm.communication_type, comm.id)
            .await
            .unwrap_or_default();

        let existing_concept_ids: std::collections::HashSet<Uuid> =
            existing.iter().map(|c| c.concept_id).collect();

        let now = chrono::Utc::now();
        let mut created = Vec::new();

        for detection in detections {
            if existing_concept_ids.contains(&detection.concept_id) {
                continue;
            }

            let cc = CommunicationConcept {
                id: Uuid::new_v4(),
                communication_type: comm.communication_type.clone(),
                communication_id: comm.id,
                concept_id: detection.concept_id,
                detection_method: DetectionMethod::Rule,
                status: SuggestionStatus::Suggested,
                confidence: Some(detection.confidence),
                reviewed_at: None,
                reviewed_by: None,
                created_at: now,
            };

            if cc_repo.create(&cc).await.is_ok() {
                created.push(cc);
            }
        }

        Ok(created)
    }

    /// Run both keyword + rule detection, deduplicate by concept_id
    pub async fn full_scan(
        &self,
        comm: &CommunicationFields,
    ) -> Result<Vec<CommunicationConcept>, String> {
        // Build text from communication fields for keyword detection
        let mut text_parts = Vec::new();
        if let Some(subj) = &comm.subject {
            text_parts.push(subj.as_str());
        }
        if let Some(body) = &comm.body {
            text_parts.push(body.as_str());
        }
        let text = text_parts.join(" ");

        // Run keyword detection
        let mut results = if !text.trim().is_empty() {
            self.scan_and_store(&comm.communication_type, comm.id, &text)
                .await?
        } else {
            Vec::new()
        };

        // Run rule-based detection (may add additional concept matches)
        let rule_results = self.scan_and_store_rules(comm).await?;
        results.extend(rule_results);

        Ok(results)
    }

    /// Retroactive: scan ALL communications for a specific concept's matchers
    pub async fn scan_all_for_concept(&self, concept_id: Uuid) -> Result<usize, String> {
        let matcher_repo = ConceptMatcherRepository::new(self.pool);
        let groups = matcher_repo
            .list_groups_by_concept(concept_id)
            .await
            .map_err(|e| format!("Failed to load matcher groups: {}", e))?;

        if groups.is_empty() {
            return Ok(0);
        }

        let concept_repo = ConceptRepository::new(self.pool);
        let concept = concept_repo
            .get_by_id(concept_id)
            .await
            .map_err(|e| format!("Failed to load concept: {}", e))?;

        let cc_repo = CommunicationConceptRepository::new(self.pool);
        let email_repo = EmailHistoryRepository::new(self.pool);
        let sms_repo = SmsHistoryRepository::new(self.pool);
        let now = chrono::Utc::now();

        let mut new_suggestions = 0usize;

        // Scan all emails
        let emails = email_repo
            .list_all()
            .await
            .map_err(|e| format!("Failed to list emails: {}", e))?;

        for email in &emails {
            let comm = CommunicationFields::from_email(email);
            let matched = evaluate_concept_against_comm(&groups, &comm);
            if !matched {
                continue;
            }

            // Check if already linked
            let existing = cc_repo
                .list_by_communication("email", email.id)
                .await
                .unwrap_or_default();
            if existing.iter().any(|c| c.concept_id == concept_id) {
                continue;
            }

            let cc = CommunicationConcept {
                id: Uuid::new_v4(),
                communication_type: "email".to_string(),
                communication_id: email.id,
                concept_id,
                detection_method: DetectionMethod::Rule,
                status: SuggestionStatus::Suggested,
                confidence: Some(0.9),
                reviewed_at: None,
                reviewed_by: None,
                created_at: now,
            };

            if cc_repo.create(&cc).await.is_ok() {
                new_suggestions += 1;
            }
        }

        // Scan all SMS
        let sms_messages = sms_repo
            .list_all()
            .await
            .map_err(|e| format!("Failed to list SMS: {}", e))?;

        for sms in &sms_messages {
            let comm = CommunicationFields::from_sms(sms);
            let matched = evaluate_concept_against_comm(&groups, &comm);
            if !matched {
                continue;
            }

            let existing = cc_repo
                .list_by_communication("sms", sms.id)
                .await
                .unwrap_or_default();
            if existing.iter().any(|c| c.concept_id == concept_id) {
                continue;
            }

            let cc = CommunicationConcept {
                id: Uuid::new_v4(),
                communication_type: "sms".to_string(),
                communication_id: sms.id,
                concept_id,
                detection_method: DetectionMethod::Rule,
                status: SuggestionStatus::Suggested,
                confidence: Some(0.9),
                reviewed_at: None,
                reviewed_by: None,
                created_at: now,
            };

            if cc_repo.create(&cc).await.is_ok() {
                new_suggestions += 1;
            }
        }

        // Suppress unused variable warning
        let _ = &concept;

        Ok(new_suggestions)
    }

    /// Retroactive: scan ALL communications for ALL concepts with matchers
    pub async fn scan_all(&self) -> Result<usize, String> {
        let matcher_repo = ConceptMatcherRepository::new(self.pool);
        let concepts = matcher_repo
            .load_all_concepts_with_matchers()
            .await
            .map_err(|e| format!("Failed to load concepts with matchers: {}", e))?;

        let mut total = 0usize;
        for cwm in &concepts {
            let count = self.scan_all_for_concept(cwm.concept_id).await?;
            total += count;
        }

        Ok(total)
    }
}

/// Evaluate a concept's matcher groups against a communication (DNF logic)
fn evaluate_concept_against_comm(
    groups: &[core_domain::ConceptMatcherGroup],
    comm: &CommunicationFields,
) -> bool {
    for group in groups {
        if group.matchers.is_empty() {
            continue;
        }
        let group_passed = group
            .matchers
            .iter()
            .all(|matcher| evaluate_matcher(matcher, comm));
        if group_passed {
            return true;
        }
    }
    false
}

/// Evaluate a single matcher against communication fields
fn evaluate_matcher(matcher: &ConceptMatcher, comm: &CommunicationFields) -> bool {
    let field_values = get_field_values(&matcher.element_type, comm);

    let raw_result = field_values.iter().any(|val| {
        match_value(
            val,
            &matcher.match_value,
            &matcher.match_mode,
            matcher.case_sensitive,
        )
    });

    if matcher.negate {
        !raw_result
    } else {
        raw_result
    }
}

/// Extract field values from communication based on element type
fn get_field_values(element_type: &MatcherElementType, comm: &CommunicationFields) -> Vec<String> {
    match element_type {
        MatcherElementType::Sender => vec![comm.sender.clone()],
        MatcherElementType::Receiver => comm.receivers.clone(),
        MatcherElementType::Subject => comm
            .subject
            .as_ref()
            .map(|s| vec![s.clone()])
            .unwrap_or_default(),
        MatcherElementType::Body => comm
            .body
            .as_ref()
            .map(|s| vec![s.clone()])
            .unwrap_or_default(),
        MatcherElementType::AttachmentName => comm.attachment_names.clone(),
        MatcherElementType::AnyText => {
            let mut vals = vec![comm.sender.clone()];
            vals.extend(comm.receivers.clone());
            if let Some(s) = &comm.subject {
                vals.push(s.clone());
            }
            if let Some(b) = &comm.body {
                vals.push(b.clone());
            }
            vals.extend(comm.attachment_names.clone());
            vals
        }
    }
}

/// Apply match mode to compare value against pattern
fn match_value(value: &str, pattern: &str, mode: &MatchMode, case_sensitive: bool) -> bool {
    let (val, pat) = if case_sensitive {
        (value.to_string(), pattern.to_string())
    } else {
        (value.to_lowercase(), pattern.to_lowercase())
    };

    match mode {
        MatchMode::Contains => val.contains(&pat),
        MatchMode::Exact => val == pat,
        MatchMode::StartsWith => val.starts_with(&pat),
        MatchMode::EndsWith => val.ends_with(&pat),
    }
}

/// Check if a keyword is found in text using case-insensitive word boundary matching.
///
/// A keyword matches if it appears as a complete word or phrase in the text.
/// For multi-word keywords, the entire phrase must be present.
fn keyword_matches(text: &str, keyword: &str) -> bool {
    let text_lower = text.to_lowercase();
    let kw_lower = keyword.to_lowercase();

    // Find the keyword in text
    let mut start = 0;
    while let Some(pos) = text_lower[start..].find(&kw_lower) {
        let abs_pos = start + pos;
        let end_pos = abs_pos + kw_lower.len();

        // Check word boundaries
        let start_ok = abs_pos == 0 || !text_lower.as_bytes()[abs_pos - 1].is_ascii_alphanumeric();
        let end_ok =
            end_pos >= text_lower.len() || !text_lower.as_bytes()[end_pos].is_ascii_alphanumeric();

        if start_ok && end_ok {
            return true;
        }

        start = abs_pos + 1;
    }

    false
}

/// Calculate a confidence score based on keyword match quality.
///
/// Longer keywords get higher confidence (more specific matches).
/// Exact case match gets a small bonus.
fn calculate_confidence(text_lower: &str, keyword: &str) -> f32 {
    let kw_lower = keyword.to_lowercase();
    let word_count = kw_lower.split_whitespace().count();

    // Base confidence: longer/more specific keywords = higher confidence
    let base = match word_count {
        1 => {
            if kw_lower.len() <= 3 {
                0.3 // Short single words are weak signals
            } else if kw_lower.len() <= 6 {
                0.5
            } else {
                0.6
            }
        }
        2 => 0.75,
        _ => 0.85, // 3+ word phrases are very specific
    };

    // Bonus: count how many times the keyword appears
    let occurrences = text_lower.matches(&kw_lower).count();
    let repetition_bonus = if occurrences > 1 {
        (occurrences as f32 * 0.05).min(0.1)
    } else {
        0.0
    };

    (base + repetition_bonus).min(0.95)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_matches_basic() {
        assert!(keyword_matches("i collect military items", "military"));
        assert!(keyword_matches("military items for sale", "military"));
        assert!(keyword_matches("items: military", "military"));
    }

    #[test]
    fn test_keyword_matches_case_insensitive() {
        assert!(keyword_matches("i have trench art", "Trench Art"));
        assert!(keyword_matches("MILITARY SURPLUS", "military"));
    }

    #[test]
    fn test_keyword_matches_word_boundaries() {
        // Should NOT match partial words
        assert!(!keyword_matches("paramilitary forces", "military"));
        assert!(!keyword_matches("antimilitarism", "military"));
        // Should match with punctuation boundaries
        assert!(keyword_matches("military, navy, and more", "military"));
        assert!(keyword_matches("(military)", "military"));
    }

    #[test]
    fn test_keyword_matches_multi_word() {
        assert!(keyword_matches(
            "i found some trench art at a sale",
            "trench art"
        ));
        assert!(!keyword_matches(
            "trench coat and art supplies",
            "trench art"
        ));
    }

    #[test]
    fn test_keyword_no_match() {
        assert!(!keyword_matches("hello world", "military"));
        assert!(!keyword_matches("", "military"));
    }

    #[test]
    fn test_calculate_confidence() {
        // Short keyword = lower confidence
        let c1 = calculate_confidence("got art", "art");
        assert!(c1 < 0.5);

        // Single word, medium length
        let c2 = calculate_confidence("military items", "military");
        assert!(c2 >= 0.5 && c2 <= 0.7);

        // Multi-word = higher confidence
        let c3 = calculate_confidence("found trench art", "trench art");
        assert!(c3 >= 0.7);

        // Very specific multi-word
        let c4 = calculate_confidence("world war ii medal", "world war ii");
        assert!(c4 >= 0.8);
    }

    #[test]
    fn test_calculate_confidence_repetition_bonus() {
        let c1 = calculate_confidence("military items", "military");
        let c2 = calculate_confidence("military items, military surplus", "military");
        assert!(c2 > c1);
    }

    #[test]
    fn test_match_value_contains() {
        assert!(match_value(
            "hello world",
            "world",
            &MatchMode::Contains,
            false
        ));
        assert!(!match_value(
            "hello world",
            "xyz",
            &MatchMode::Contains,
            false
        ));
    }

    #[test]
    fn test_match_value_exact() {
        assert!(match_value("hello", "hello", &MatchMode::Exact, false));
        assert!(match_value("Hello", "hello", &MatchMode::Exact, false));
        assert!(!match_value(
            "hello world",
            "hello",
            &MatchMode::Exact,
            false
        ));
    }

    #[test]
    fn test_match_value_starts_with() {
        assert!(match_value(
            "hello world",
            "hello",
            &MatchMode::StartsWith,
            false
        ));
        assert!(!match_value(
            "hello world",
            "world",
            &MatchMode::StartsWith,
            false
        ));
    }

    #[test]
    fn test_match_value_ends_with() {
        assert!(match_value(
            "hello world",
            "world",
            &MatchMode::EndsWith,
            false
        ));
        assert!(!match_value(
            "hello world",
            "hello",
            &MatchMode::EndsWith,
            false
        ));
    }

    #[test]
    fn test_match_value_case_sensitive() {
        assert!(!match_value("Hello", "hello", &MatchMode::Exact, true));
        assert!(match_value("Hello", "Hello", &MatchMode::Exact, true));
    }

    #[test]
    fn test_evaluate_matcher_negate() {
        let comm = CommunicationFields {
            id: Uuid::new_v4(),
            communication_type: "email".to_string(),
            sender: "spam@example.com".to_string(),
            receivers: vec!["me@example.com".to_string()],
            subject: Some("Buy now!".to_string()),
            body: None,
            attachment_names: Vec::new(),
        };

        // NOT sender contains "spam" should NOT match (because sender does contain spam)
        let matcher = ConceptMatcher {
            id: Uuid::new_v4(),
            group_id: Uuid::new_v4(),
            element_type: MatcherElementType::Sender,
            match_value: "spam".to_string(),
            match_mode: MatchMode::Contains,
            negate: true,
            case_sensitive: false,
            position: 0,
            created_at: chrono::Utc::now(),
        };
        assert!(!evaluate_matcher(&matcher, &comm));

        // NOT sender contains "trusted" should match (because sender doesn't contain trusted)
        let matcher2 = ConceptMatcher {
            id: Uuid::new_v4(),
            group_id: Uuid::new_v4(),
            element_type: MatcherElementType::Sender,
            match_value: "trusted".to_string(),
            match_mode: MatchMode::Contains,
            negate: true,
            case_sensitive: false,
            position: 0,
            created_at: chrono::Utc::now(),
        };
        assert!(evaluate_matcher(&matcher2, &comm));
    }

    #[test]
    fn test_evaluate_matcher_any_text() {
        let comm = CommunicationFields {
            id: Uuid::new_v4(),
            communication_type: "email".to_string(),
            sender: "alice@example.com".to_string(),
            receivers: vec!["bob@example.com".to_string()],
            subject: Some("Invoice #1234".to_string()),
            body: Some("Please find attached invoice".to_string()),
            attachment_names: vec!["invoice.pdf".to_string()],
        };

        let matcher = ConceptMatcher {
            id: Uuid::new_v4(),
            group_id: Uuid::new_v4(),
            element_type: MatcherElementType::AnyText,
            match_value: "invoice".to_string(),
            match_mode: MatchMode::Contains,
            negate: false,
            case_sensitive: false,
            position: 0,
            created_at: chrono::Utc::now(),
        };
        assert!(evaluate_matcher(&matcher, &comm));
    }
}
