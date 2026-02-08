use ai_middleware::SegmindClient;
use chrono::Utc;
use core_domain::{CommunicationConcept, Concept, ConceptKeyword, DetectionMethod, SuggestionStatus};
use local_store::repositories::{
    CommunicationConceptRepository, ConceptRepository, EmailHistoryRepository, EmailMessage,
    EmailTriageSessionRepository, EmailTriageSession,
};
use local_store::DbPool;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

/// Result of analyzing a block of emails for domain discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriageBlockResult {
    pub session_id: String,
    pub block_number: i64,
    pub emails_analyzed: usize,
    pub proposals: Vec<DomainProposal>,
}

/// A proposed communication domain discovered from email analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainProposal {
    pub concept_id: String,
    pub name: String,
    pub description: String,
    pub keywords: Vec<String>,
    pub email_count: usize,
    pub sample_subjects: Vec<String>,
    pub sample_senders: Vec<String>,
    pub confidence: f32,
}

/// Domain stats for the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainStats {
    pub concept_id: String,
    pub name: String,
    pub email_count: i64,
    pub unique_senders: i64,
    pub earliest_date: Option<i64>,
    pub latest_date: Option<i64>,
    pub top_senders: Vec<SenderCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SenderCount {
    pub address: String,
    pub name: Option<String>,
    pub count: i64,
}

pub struct EmailTriageService {
    pool: Arc<DbPool>,
    ai_client: SegmindClient,
}

impl EmailTriageService {
    pub fn new(pool: &Arc<DbPool>, ai_client: SegmindClient) -> Self {
        Self { pool: pool.clone(), ai_client }
    }

    /// Start a new triage session: creates the session record
    pub async fn start_session(
        &self,
        user_id: Uuid,
        server: &str,
        folder: &str,
        block_size: i64,
    ) -> Result<EmailTriageSession, String> {
        let email_repo = EmailHistoryRepository::new(&self.pool);
        let session_repo = EmailTriageSessionRepository::new(&self.pool);

        let total = email_repo.count().await.map_err(|e| e.to_string())?;
        let now = Utc::now();

        let session = EmailTriageSession {
            id: Uuid::new_v4(),
            user_id,
            server: server.to_string(),
            folder: folder.to_string(),
            status: "analyzing".to_string(),
            total_fetched: total,
            total_analyzed: 0,
            total_categorized: 0,
            current_block: 0,
            block_size,
            discovered_domains: "[]".to_string(),
            approved_domains: "[]".to_string(),
            denied_domains: "[]".to_string(),
            error_message: None,
            created_at: now,
            updated_at: now,
            completed_at: None,
        };

        session_repo
            .create(&session)
            .await
            .map_err(|e| e.to_string())?;

        info!(
            "Started triage session {} for {} emails",
            session.id, total
        );

        Ok(session)
    }

    /// Analyze a block of emails and propose communication domains
    pub async fn analyze_block(
        &self,
        session_id: Uuid,
        block_number: i64,
        block_size: i64,
        existing_approved_domains: &[String], // concept IDs already approved
        user_id: Uuid,
    ) -> Result<TriageBlockResult, String> {
        let email_repo = EmailHistoryRepository::new(&self.pool);
        let concept_repo = ConceptRepository::new(&self.pool);
        let session_repo = EmailTriageSessionRepository::new(&self.pool);

        let offset = block_number * block_size;
        let emails = email_repo
            .get_block(offset, block_size)
            .await
            .map_err(|e| e.to_string())?;

        if emails.is_empty() {
            return Ok(TriageBlockResult {
                session_id: session_id.to_string(),
                block_number,
                emails_analyzed: 0,
                proposals: Vec::new(),
            });
        }

        // Get existing approved domain names for context
        let mut approved_names = Vec::new();
        for domain_id_str in existing_approved_domains {
            if let Ok(uid) = Uuid::parse_str(domain_id_str) {
                if let Ok(concept) = concept_repo.get_by_id(uid).await {
                    approved_names.push(concept.name.clone());
                }
            }
        }

        // Try AI analysis, fall back to rule-based grouping
        let proposals = match self.ai_discover_domains(&emails, &approved_names).await {
            Ok(p) => p,
            Err(e) => {
                warn!("AI domain discovery failed, using rule-based fallback: {}", e);
                self.rule_based_discover_domains(&emails)
            }
        };

        // Create Concept records for each proposal and CommunicationConcept links
        let mut final_proposals = Vec::new();
        for proposal in proposals {
            let concept_id = self
                .create_domain_concept(&proposal, session_id, user_id)
                .await?;

            // Create CommunicationConcept links for the emails in this proposal
            self.link_emails_to_concept(&emails, &proposal, concept_id)
                .await?;

            final_proposals.push(DomainProposal {
                concept_id: concept_id.to_string(),
                ..proposal
            });
        }

        // Update session progress
        let total_analyzed = (block_number + 1) * block_size;
        session_repo
            .update_progress(
                session_id,
                -1, // don't change total_fetched
                total_analyzed.min(email_repo.count().await.map_err(|e| e.to_string())?),
                0,
                block_number,
            )
            .await
            .map_err(|e| e.to_string())?;

        Ok(TriageBlockResult {
            session_id: session_id.to_string(),
            block_number,
            emails_analyzed: emails.len(),
            proposals: final_proposals,
        })
    }

    /// AI-powered domain discovery from a block of emails
    async fn ai_discover_domains(
        &self,
        emails: &[EmailMessage],
        approved_domain_names: &[String],
    ) -> Result<Vec<DomainProposal>, String> {
        // Build the prompt
        let mut email_summaries = String::new();
        for (i, email) in emails.iter().enumerate() {
            let body_preview = email
                .body_text
                .as_ref()
                .map(|b| {
                    if b.len() > 100 {
                        let mut e = 100;
                        while !b.is_char_boundary(e) { e -= 1; }
                        format!("{}...", &b[..e])
                    } else {
                        b.clone()
                    }
                })
                .unwrap_or_default();

            email_summaries.push_str(&format!(
                "Email {}: From: {} <{}>, Subject: \"{}\", Preview: \"{}\"\n",
                i, email.from_name.as_deref().unwrap_or(""), email.from_address, email.subject, body_preview
            ));
        }

        let approved_list = if approved_domain_names.is_empty() {
            "None yet".to_string()
        } else {
            approved_domain_names.join(", ")
        };

        let prompt = format!(
            r#"Analyze these emails and group them into Communication Domains.
A Communication Domain is a category like "Personal", "Bills & Receipts", "Work", "Newsletters", "Shopping", "Social Media Notifications", etc.

Existing approved domains (reuse when appropriate): {}

Emails:
{}

Return ONLY a JSON array (no other text) of proposed domains:
[{{"name": "Domain Name", "description": "What this domain represents", "keywords": ["keyword1", "keyword2"], "email_indices": [0, 3, 5], "confidence": 0.85}}]

Rules:
- Group related emails together
- Reuse existing approved domain names when emails fit
- Each email can belong to multiple domains
- Confidence 0.0-1.0 based on how clearly emails fit the domain
- Use descriptive, user-friendly domain names"#,
            approved_list, email_summaries
        );

        let response = self
            .ai_client
            .generate_suggestion(&prompt)
            .await
            .map_err(|e| format!("AI call failed: {}", e))?;

        // Parse the JSON response
        let text = response.text.trim();
        // Try to extract JSON array from the response
        let json_start = text.find('[').ok_or("No JSON array in AI response")?;
        let json_end = text.rfind(']').ok_or("No closing bracket in AI response")?;
        let json_str = &text[json_start..=json_end];

        let raw_proposals: Vec<RawAiProposal> =
            serde_json::from_str(json_str).map_err(|e| format!("Failed to parse AI response: {}", e))?;

        // Convert to DomainProposal
        let proposals = raw_proposals
            .into_iter()
            .map(|raw| {
                let sample_subjects: Vec<String> = raw
                    .email_indices
                    .iter()
                    .filter_map(|&idx| emails.get(idx).map(|e| e.subject.clone()))
                    .take(3)
                    .collect();
                let sample_senders: Vec<String> = raw
                    .email_indices
                    .iter()
                    .filter_map(|&idx| emails.get(idx).map(|e| e.from_address.clone()))
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .take(3)
                    .collect();

                DomainProposal {
                    concept_id: String::new(), // will be set when concept is created
                    name: raw.name,
                    description: raw.description,
                    keywords: raw.keywords,
                    email_count: raw.email_indices.len(),
                    sample_subjects,
                    sample_senders,
                    confidence: raw.confidence,
                }
            })
            .collect();

        Ok(proposals)
    }

    /// Rule-based fallback: group emails by sender domain
    fn rule_based_discover_domains(&self, emails: &[EmailMessage]) -> Vec<DomainProposal> {
        let mut domain_groups: HashMap<String, Vec<usize>> = HashMap::new();

        for (i, email) in emails.iter().enumerate() {
            let sender_domain = email
                .from_address
                .split('@')
                .nth(1)
                .unwrap_or("unknown")
                .to_lowercase();

            domain_groups
                .entry(sender_domain)
                .or_default()
                .push(i);
        }

        domain_groups
            .into_iter()
            .filter(|(_, indices)| indices.len() >= 2) // Need at least 2 emails
            .map(|(domain, indices)| {
                let name = match domain.as_str() {
                    "gmail.com" => "Personal Gmail".to_string(),
                    "yahoo.com" => "Yahoo Mail".to_string(),
                    "outlook.com" | "hotmail.com" => "Outlook/Hotmail".to_string(),
                    d if d.contains("ebay") => "eBay".to_string(),
                    d if d.contains("amazon") => "Amazon".to_string(),
                    d if d.contains("paypal") => "PayPal".to_string(),
                    d if d.contains("facebook") || d.contains("fb") => "Facebook".to_string(),
                    d if d.contains("google") => "Google Services".to_string(),
                    d => format!("From @{}", d),
                };

                let sample_subjects: Vec<String> = indices
                    .iter()
                    .filter_map(|&idx| emails.get(idx).map(|e| e.subject.clone()))
                    .take(3)
                    .collect();
                let sample_senders: Vec<String> = indices
                    .iter()
                    .filter_map(|&idx| emails.get(idx).map(|e| e.from_address.clone()))
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .take(3)
                    .collect();

                DomainProposal {
                    concept_id: String::new(),
                    name,
                    description: format!("Emails from @{}", domain),
                    keywords: vec![format!("@{}", domain), domain.clone()],
                    email_count: indices.len(),
                    sample_subjects,
                    sample_senders,
                    confidence: 0.6,
                }
            })
            .collect()
    }

    /// Create a Concept record for a proposed domain
    async fn create_domain_concept(
        &self,
        proposal: &DomainProposal,
        _session_id: Uuid,
        user_id: Uuid,
    ) -> Result<Uuid, String> {
        let concept_repo = ConceptRepository::new(&self.pool);

        // Check if a domain with this name already exists
        let existing = concept_repo.search(&proposal.name).await.map_err(|e| e.to_string())?;
        for c in &existing {
            if c.name.to_lowercase() == proposal.name.to_lowercase() {
                // Check if it's already a communication_domain
                if let Some(ct) = c.metadata.get("concept_type") {
                    if ct.as_str() == Some("communication_domain") {
                        return Ok(c.id);
                    }
                }
            }
        }

        let concept_id = Uuid::new_v4();
        let now = Utc::now();

        let keywords: Vec<ConceptKeyword> = proposal
            .keywords
            .iter()
            .map(|kw| ConceptKeyword {
                id: Uuid::new_v4(),
                concept_id,
                keyword: kw.clone(),
                created_at: now,
            })
            .collect();

        let concept = Concept {
            id: concept_id,
            name: proposal.name.clone(),
            description: Some(proposal.description.clone()),
            parent_id: None,
            keywords,
            metadata: serde_json::json!({
                "concept_type": "communication_domain",
                "confidence": proposal.confidence,
                "auto_discovered": true,
            }),
            created_at: now,
            updated_at: now,
            created_by: user_id,
        };

        concept_repo
            .create(&concept)
            .await
            .map_err(|e| format!("Failed to create domain concept: {}", e))?;

        info!("Created communication domain concept: {} ({})", proposal.name, concept_id);

        Ok(concept_id)
    }

    /// Link emails in a proposal to their concept via communication_concepts
    async fn link_emails_to_concept(
        &self,
        emails: &[EmailMessage],
        proposal: &DomainProposal,
        concept_id: Uuid,
    ) -> Result<(), String> {
        let cc_repo = CommunicationConceptRepository::new(&self.pool);
        let now = Utc::now();

        // Link all emails that match this domain's keywords
        for email in emails {
            let text = format!(
                "{} {} {}",
                email.subject,
                email.from_address,
                email.body_text.as_deref().unwrap_or("")
            )
            .to_lowercase();

            let matches = proposal.keywords.iter().any(|kw| text.contains(&kw.to_lowercase()));
            if !matches {
                continue;
            }

            // Check if link already exists
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
                detection_method: DetectionMethod::Ai,
                status: SuggestionStatus::Suggested,
                confidence: Some(proposal.confidence),
                reviewed_at: None,
                reviewed_by: None,
                created_at: now,
            };

            if let Err(e) = cc_repo.create(&cc).await {
                warn!("Failed to link email {} to concept {}: {}", email.id, concept_id, e);
            }
        }

        Ok(())
    }

    /// After user approves domains, apply their keywords to categorize remaining emails
    pub async fn apply_approved_domains(
        &self,
        session_id: Uuid,
        approved_concept_ids: &[Uuid],
    ) -> Result<i64, String> {
        let email_repo = EmailHistoryRepository::new(&self.pool);
        let concept_repo = ConceptRepository::new(&self.pool);
        let cc_repo = CommunicationConceptRepository::new(&self.pool);
        let session_repo = EmailTriageSessionRepository::new(&self.pool);

        let now = Utc::now();
        let mut total_categorized: i64 = 0;

        // Load approved domain concepts with their keywords
        let mut domains: Vec<(Uuid, Vec<String>)> = Vec::new();
        for &concept_id in approved_concept_ids {
            if let Ok(concept) = concept_repo.get_by_id(concept_id).await {
                let kws: Vec<String> = concept.keywords.iter().map(|k| k.keyword.to_lowercase()).collect();
                domains.push((concept_id, kws));
            }
        }

        // Process all emails in batches
        let total = email_repo.count().await.map_err(|e| e.to_string())?;
        let batch_size: i64 = 100;
        let mut offset: i64 = 0;

        while offset < total {
            let emails = email_repo
                .get_block(offset, batch_size)
                .await
                .map_err(|e| e.to_string())?;

            if emails.is_empty() {
                break;
            }

            for email in &emails {
                let text = format!(
                    "{} {} {}",
                    email.subject,
                    email.from_address,
                    email.body_text.as_deref().unwrap_or("")
                )
                .to_lowercase();

                for (concept_id, keywords) in &domains {
                    let matches = keywords.iter().any(|kw| text.contains(kw));
                    if !matches {
                        continue;
                    }

                    // Check if already linked
                    let existing = cc_repo
                        .list_by_communication("email", email.id)
                        .await
                        .unwrap_or_default();

                    if existing.iter().any(|c| c.concept_id == *concept_id) {
                        continue;
                    }

                    let cc = CommunicationConcept {
                        id: Uuid::new_v4(),
                        communication_type: "email".to_string(),
                        communication_id: email.id,
                        concept_id: *concept_id,
                        detection_method: DetectionMethod::Keyword,
                        status: SuggestionStatus::Confirmed,
                        confidence: Some(0.7),
                        reviewed_at: Some(now),
                        reviewed_by: None,
                        created_at: now,
                    };

                    if cc_repo.create(&cc).await.is_ok() {
                        total_categorized += 1;
                    }
                }
            }

            offset += batch_size;
        }

        // Update session
        session_repo
            .update_progress(session_id, -1, -1, total_categorized, -1)
            .await
            .ok(); // best-effort

        info!(
            "Applied approved domains to {} email-concept links",
            total_categorized
        );

        Ok(total_categorized)
    }

    /// Get domain stats for a specific communication domain concept
    pub async fn get_domain_stats(&self, concept_id: Uuid) -> Result<DomainStats, String> {
        let concept_repo = ConceptRepository::new(&self.pool);
        let concept = concept_repo
            .get_by_id(concept_id)
            .await
            .map_err(|e| e.to_string())?;

        // Get email count + date range
        let stats_row = sqlx::query_as::<_, StatsRow>(
            "SELECT COUNT(DISTINCT e.id) as email_count,
             COUNT(DISTINCT e.from_address) as unique_senders,
             MIN(e.message_date) as earliest_date,
             MAX(e.message_date) as latest_date
             FROM email_history e
             INNER JOIN communication_concepts cc ON cc.communication_id = e.id
                AND cc.communication_type = 'email'
             WHERE cc.concept_id = ? AND cc.status IN ('confirmed', 'suggested')",
        )
        .bind(concept_id.to_string())
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| format!("Failed to get domain stats: {}", e))?;

        // Get top senders
        let sender_rows = sqlx::query_as::<_, SenderRow>(
            "SELECT e.from_address as address, e.from_name as name, COUNT(*) as count
             FROM email_history e
             INNER JOIN communication_concepts cc ON cc.communication_id = e.id
                AND cc.communication_type = 'email'
             WHERE cc.concept_id = ? AND cc.status IN ('confirmed', 'suggested')
             GROUP BY e.from_address
             ORDER BY count DESC
             LIMIT 10",
        )
        .bind(concept_id.to_string())
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| format!("Failed to get top senders: {}", e))?;

        Ok(DomainStats {
            concept_id: concept_id.to_string(),
            name: concept.name,
            email_count: stats_row.email_count,
            unique_senders: stats_row.unique_senders,
            earliest_date: stats_row.earliest_date,
            latest_date: stats_row.latest_date,
            top_senders: sender_rows
                .into_iter()
                .map(|r| SenderCount {
                    address: r.address,
                    name: r.name,
                    count: r.count,
                })
                .collect(),
        })
    }
}

#[derive(Debug, Deserialize)]
struct RawAiProposal {
    name: String,
    description: String,
    keywords: Vec<String>,
    email_indices: Vec<usize>,
    confidence: f32,
}

#[derive(Debug, sqlx::FromRow)]
struct StatsRow {
    email_count: i64,
    unique_senders: i64,
    earliest_date: Option<i64>,
    latest_date: Option<i64>,
}

#[derive(Debug, sqlx::FromRow)]
struct SenderRow {
    address: String,
    name: Option<String>,
    count: i64,
}
