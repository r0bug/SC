use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

/// AI analysis result for an email
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAnalysis {
    pub importance_score: i32,  // 0-10
    pub is_important: bool,      // score >= 7
    pub is_urgent: bool,
    pub summary: String,
    pub reasoning: String,
}

/// Email AI analyzer using mock or real AI
pub struct EmailAIAnalyzer {
    db_pool: Arc<sqlx::SqlitePool>,
    use_real_ai: bool,
}

impl EmailAIAnalyzer {
    pub fn new(db_pool: Arc<sqlx::SqlitePool>) -> Self {
        // Check if AI is available (via SEGMIND_API_KEY or other AI service)
        let use_real_ai = std::env::var("SEGMIND_API_KEY").is_ok()
            || std::env::var("OPENAI_API_KEY").is_ok();

        if use_real_ai {
            info!("✅ Email AI analysis enabled");
        } else {
            info!("ℹ️ Using mock AI analysis (set AI API key for real analysis)");
        }

        Self {
            db_pool,
            use_real_ai,
        }
    }

    /// Analyze an email for importance
    pub async fn analyze_email(
        &self,
        email_id: Uuid,
        from_address: &str,
        subject: &str,
        body: &str,
    ) -> Result<EmailAnalysis> {
        info!("🤖 Analyzing email importance: {}", subject);

        let analysis = if self.use_real_ai {
            self.analyze_with_ai(from_address, subject, body).await?
        } else {
            self.analyze_with_rules(from_address, subject, body)
        };

        // Store analysis in database
        // Create let bindings for temporary values to extend their lifetime
        let is_important_int = analysis.is_important as i32;
        let is_urgent_int = analysis.is_urgent as i32;
        let ai_analysis_json = serde_json::to_string(&analysis).unwrap();
        let analyzed_at = chrono::Utc::now().to_rfc3339();
        let email_id_str = email_id.to_string();

        sqlx::query!(
            r#"
            UPDATE email_history
            SET importance_score = ?,
                is_important = ?,
                is_urgent = ?,
                ai_summary = ?,
                ai_analysis = ?,
                ai_analyzed_at = ?
            WHERE id = ?
            "#,
            analysis.importance_score,
            is_important_int,
            is_urgent_int,
            analysis.summary,
            ai_analysis_json,
            analyzed_at,
            email_id_str
        )
        .execute(self.db_pool.as_ref())
        .await?;

        if analysis.is_important {
            info!("⚠️ Important email detected (score: {})", analysis.importance_score);
        }

        Ok(analysis)
    }

    /// Analyze with real AI (placeholder for AI integration)
    async fn analyze_with_ai(
        &self,
        from_address: &str,
        subject: &str,
        body: &str,
    ) -> Result<EmailAnalysis> {
        // TODO: Integrate with ai_middleware crate for real AI analysis
        // For now, use rule-based as fallback
        warn!("Real AI analysis not yet implemented, using rules");
        Ok(self.analyze_with_rules(from_address, subject, body))
    }

    /// Rule-based importance analysis (fallback)
    fn analyze_with_rules(
        &self,
        from_address: &str,
        subject: &str,
        body: &str,
    ) -> EmailAnalysis {
        let mut score: i32 = 5; // Baseline

        let subject_lower = subject.to_lowercase();
        let body_lower = body.to_lowercase();

        // Urgency indicators
        let urgent_keywords = [
            "urgent", "asap", "immediately", "emergency", "critical",
            "deadline", "today", "now", "important", "action required"
        ];

        let is_urgent = urgent_keywords.iter().any(|&keyword| {
            subject_lower.contains(keyword) || body_lower.contains(keyword)
        });

        if is_urgent {
            score += 3;
        }

        // Priority indicators in subject
        if subject_lower.contains("[priority]") || subject_lower.contains("priority:") {
            score += 2;
        }

        // Questions (usually require response)
        if subject_lower.contains('?') || body_lower.matches('?').count() > 2 {
            score += 1;
        }

        // Direct address (personalized)
        if body_lower.contains("you") || body_lower.contains("your") {
            score += 1;
        }

        // Known important senders (VIP domains)
        let vip_domains = ["ceo@", "founder@", "admin@", "security@", "billing@"];
        if vip_domains.iter().any(|&domain| from_address.contains(domain)) {
            score += 2;
        }

        // Spam indicators (reduce score)
        let spam_keywords = ["unsubscribe", "click here", "winner", "congratulations", "prize"];
        if spam_keywords.iter().any(|&keyword| body_lower.contains(keyword)) {
            score = score.saturating_sub(3);
        }

        // Cap score at 10
        score = score.min(10);

        // Generate summary (first 100 chars of body)
        let summary = body.chars().take(100).collect::<String>()
            + if body.len() > 100 { "..." } else { "" };

        EmailAnalysis {
            importance_score: score,
            is_important: score >= 7,
            is_urgent,
            summary,
            reasoning: format!(
                "Rule-based analysis: urgency={}, score={}",
                is_urgent, score
            ),
        }
    }

    /// Batch analyze unanalyzed emails
    pub async fn analyze_pending_emails(&self) -> Result<usize> {
        // Find emails that haven't been analyzed
        let emails = sqlx::query!(
            r#"
            SELECT id, from_address, subject, body_text
            FROM email_history
            WHERE ai_analyzed_at IS NULL
            LIMIT 50
            "#
        )
        .fetch_all(self.db_pool.as_ref())
        .await?;

        let count = emails.len();
        if count == 0 {
            return Ok(0);
        }

        info!("📊 Analyzing {} pending emails...", count);

        for email in emails {
            let email_id_str = email.id.as_deref().unwrap_or("");
            let email_id = match Uuid::parse_str(email_id_str) {
                Ok(id) => id,
                Err(e) => {
                    warn!("Invalid email ID '{}': {}", email_id_str, e);
                    continue;
                }
            };
            let body = email.body_text.as_deref().unwrap_or("");

            if let Err(e) = self.analyze_email(
                email_id,
                &email.from_address,
                &email.subject,
                body,
            ).await {
                warn!("Failed to analyze email {}: {}", email_id_str, e);
            }
        }

        Ok(count)
    }
}
