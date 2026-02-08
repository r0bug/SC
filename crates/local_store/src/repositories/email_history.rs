use crate::db::DbPool;
use chrono::{DateTime, Utc};
use core_domain::{DomainError, DomainResult};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct EmailMessage {
    pub id: Uuid,
    pub contact_id: Option<Uuid>,
    pub from_address: String,
    pub from_name: Option<String>,
    pub to_address: String,
    pub cc: Option<String>,
    pub bcc: Option<String>,
    pub subject: String,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub message_date: i64, // Unix timestamp in milliseconds
    pub message_type: i32, // 1=received, 2=sent
    pub message_id: Option<String>,
    pub in_reply_to: Option<String>,
    pub email_references: Option<String>,
    pub read_status: i32, // 0=unread, 1=read
    pub has_attachments: i32,
    pub folder: Option<String>,
    pub imported_at: DateTime<Utc>,
    pub imported_by: String,
    pub source_server: Option<String>,
}

pub struct EmailHistoryRepository<'a> {
    pool: &'a DbPool,
}

impl<'a> EmailHistoryRepository<'a> {
    pub fn new(pool: &'a DbPool) -> Self {
        Self { pool }
    }

    /// Batch insert multiple email messages, skipping duplicates on message_id
    pub async fn batch_create(&self, messages: &[EmailMessage]) -> DomainResult<usize> {
        let mut tx =
            self.pool.begin().await.map_err(|e| {
                DomainError::Internal(format!("Failed to start transaction: {}", e))
            })?;

        let mut count = 0;
        for email in messages {
            let result = sqlx::query(
                "INSERT OR IGNORE INTO email_history (id, contact_id, from_address, from_name, to_address,
                 cc, bcc, subject, body_text, body_html, message_date, message_type, message_id,
                 in_reply_to, email_references, read_status, has_attachments, folder,
                 imported_at, imported_by, source_server)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(email.id.to_string())
            .bind(email.contact_id.map(|id| id.to_string()))
            .bind(&email.from_address)
            .bind(&email.from_name)
            .bind(&email.to_address)
            .bind(&email.cc)
            .bind(&email.bcc)
            .bind(&email.subject)
            .bind(&email.body_text)
            .bind(&email.body_html)
            .bind(email.message_date)
            .bind(email.message_type)
            .bind(&email.message_id)
            .bind(&email.in_reply_to)
            .bind(&email.email_references)
            .bind(email.read_status)
            .bind(email.has_attachments)
            .bind(&email.folder)
            .bind(email.imported_at.to_rfc3339())
            .bind(&email.imported_by)
            .bind(&email.source_server)
            .execute(&mut *tx)
            .await
            .map_err(|e| DomainError::Internal(format!("Failed to insert email: {}", e)))?;

            if result.rows_affected() > 0 {
                count += 1;
            }
        }

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(format!("Failed to commit transaction: {}", e)))?;

        Ok(count)
    }

    /// Check if a message_id already exists (for deduplication)
    pub async fn exists_by_message_id(&self, message_id: &str) -> DomainResult<bool> {
        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM email_history WHERE message_id = ?")
                .bind(message_id)
                .fetch_one(self.pool)
                .await
                .map_err(|e| DomainError::Internal(format!("Failed to check message_id: {}", e)))?;

        Ok(count.0 > 0)
    }

    /// Get all emails for an address (from or to)
    pub async fn get_by_address(&self, address: &str) -> DomainResult<Vec<EmailMessage>> {
        let rows = sqlx::query_as::<_, EmailMessageRow>(
            "SELECT id, contact_id, from_address, from_name, to_address, cc, bcc,
             subject, body_text, body_html, message_date, message_type, message_id,
             in_reply_to, email_references, read_status, has_attachments, folder,
             imported_at, imported_by, source_server
             FROM email_history WHERE from_address = ? OR to_address = ?
             ORDER BY message_date DESC",
        )
        .bind(address)
        .bind(address)
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(format!("Failed to fetch emails: {}", e)))?;

        rows.into_iter().map(|row| row.try_into()).collect()
    }

    /// Get conversations grouped by from_address with counts
    pub async fn get_conversations(&self) -> DomainResult<Vec<EmailConversation>> {
        let rows = sqlx::query_as::<_, EmailConversationRow>(
            r#"
            SELECT
                from_address,
                from_name,
                contact_id,
                COUNT(*) as message_count,
                MAX(message_date) as last_message_date
            FROM email_history
            WHERE message_type = 1
            GROUP BY from_address
            ORDER BY last_message_date DESC
            "#,
        )
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(format!("Failed to fetch conversations: {}", e)))?;

        Ok(rows
            .into_iter()
            .map(|r| EmailConversation {
                from_address: r.from_address,
                from_name: r.from_name,
                contact_id: r.contact_id,
                message_count: r.message_count,
                last_message_date: r.last_message_date,
            })
            .collect())
    }

    /// List all emails (for bulk operations like retroactive scanning)
    pub async fn list_all(&self) -> DomainResult<Vec<EmailMessage>> {
        let rows = sqlx::query_as::<_, EmailMessageRow>(
            "SELECT id, contact_id, from_address, from_name, to_address, cc, bcc,
             subject, body_text, body_html, message_date, message_type, message_id,
             in_reply_to, email_references, read_status, has_attachments, folder,
             imported_at, imported_by, source_server
             FROM email_history
             ORDER BY message_date DESC",
        )
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(format!("Failed to list all emails: {}", e)))?;

        rows.into_iter().map(|row| row.try_into()).collect()
    }

    /// Count total emails
    pub async fn count(&self) -> DomainResult<i64> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM email_history")
            .fetch_one(self.pool)
            .await
            .map_err(|e| DomainError::Internal(format!("Failed to count emails: {}", e)))?;

        Ok(count.0)
    }

    /// Get a paginated block of emails for triage processing
    pub async fn get_block(&self, offset: i64, limit: i64) -> DomainResult<Vec<EmailMessage>> {
        let rows = sqlx::query_as::<_, EmailMessageRow>(
            "SELECT id, contact_id, from_address, from_name, to_address, cc, bcc,
             subject, body_text, body_html, message_date, message_type, message_id,
             in_reply_to, email_references, read_status, has_attachments, folder,
             imported_at, imported_by, source_server
             FROM email_history
             ORDER BY message_date DESC
             LIMIT ? OFFSET ?",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(format!("Failed to fetch email block: {}", e)))?;

        rows.into_iter().map(|row| row.try_into()).collect()
    }

    /// Get emails that haven't been AI-analyzed yet
    pub async fn get_unanalyzed(&self, limit: i64) -> DomainResult<Vec<EmailMessage>> {
        let rows = sqlx::query_as::<_, EmailMessageRow>(
            "SELECT id, contact_id, from_address, from_name, to_address, cc, bcc,
             subject, body_text, body_html, message_date, message_type, message_id,
             in_reply_to, email_references, read_status, has_attachments, folder,
             imported_at, imported_by, source_server
             FROM email_history WHERE ai_analyzed_at IS NULL
             ORDER BY message_date DESC
             LIMIT ?",
        )
        .bind(limit)
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(format!("Failed to fetch unanalyzed emails: {}", e)))?;

        rows.into_iter().map(|row| row.try_into()).collect()
    }

    /// Store AI analysis results for an email
    pub async fn update_ai_analysis(
        &self,
        id: Uuid,
        importance_score: Option<i32>,
        is_important: bool,
        is_urgent: bool,
        ai_summary: Option<&str>,
        ai_analysis: Option<&str>,
    ) -> DomainResult<()> {
        sqlx::query(
            "UPDATE email_history SET importance_score = ?, is_important = ?, is_urgent = ?,
             ai_summary = ?, ai_analysis = ?, ai_analyzed_at = ? WHERE id = ?",
        )
        .bind(importance_score)
        .bind(is_important as i32)
        .bind(is_urgent as i32)
        .bind(ai_summary)
        .bind(ai_analysis)
        .bind(chrono::Utc::now().to_rfc3339())
        .bind(id.to_string())
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(format!("Failed to update AI analysis: {}", e)))?;

        Ok(())
    }

    /// Get emails linked to a specific concept via communication_concepts
    pub async fn get_by_concept(&self, concept_id: Uuid) -> DomainResult<Vec<EmailMessage>> {
        let rows = sqlx::query_as::<_, EmailMessageRow>(
            "SELECT e.id, e.contact_id, e.from_address, e.from_name, e.to_address, e.cc, e.bcc,
             e.subject, e.body_text, e.body_html, e.message_date, e.message_type, e.message_id,
             e.in_reply_to, e.email_references, e.read_status, e.has_attachments, e.folder,
             e.imported_at, e.imported_by, e.source_server
             FROM email_history e
             INNER JOIN communication_concepts cc ON cc.communication_id = e.id
                AND cc.communication_type = 'email'
             WHERE cc.concept_id = ? AND cc.status = 'confirmed'
             ORDER BY e.message_date DESC",
        )
        .bind(concept_id.to_string())
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(format!("Failed to fetch emails by concept: {}", e)))?;

        rows.into_iter().map(|row| row.try_into()).collect()
    }

    /// Filter emails by multiple concept IDs in union or intersection mode
    pub async fn filter_by_concepts(
        &self,
        concept_ids: &[Uuid],
        mode: &str, // "union" or "intersection"
        limit: i64,
        offset: i64,
    ) -> DomainResult<Vec<EmailWithDomains>> {
        if concept_ids.is_empty() {
            return Ok(Vec::new());
        }

        let placeholders: Vec<&str> = concept_ids.iter().map(|_| "?").collect();
        let in_clause = placeholders.join(", ");

        let query = if mode == "intersection" {
            format!(
                "SELECT e.id, e.from_address, e.from_name, e.subject, e.message_date,
                 e.body_text, e.message_type, e.folder
                 FROM email_history e
                 INNER JOIN communication_concepts cc ON cc.communication_id = e.id
                    AND cc.communication_type = 'email'
                 WHERE cc.concept_id IN ({}) AND cc.status = 'confirmed'
                 GROUP BY e.id
                 HAVING COUNT(DISTINCT cc.concept_id) = ?
                 ORDER BY e.message_date DESC
                 LIMIT ? OFFSET ?",
                in_clause
            )
        } else {
            format!(
                "SELECT DISTINCT e.id, e.from_address, e.from_name, e.subject, e.message_date,
                 e.body_text, e.message_type, e.folder
                 FROM email_history e
                 INNER JOIN communication_concepts cc ON cc.communication_id = e.id
                    AND cc.communication_type = 'email'
                 WHERE cc.concept_id IN ({}) AND cc.status = 'confirmed'
                 ORDER BY e.message_date DESC
                 LIMIT ? OFFSET ?",
                in_clause
            )
        };

        let mut q = sqlx::query_as::<_, EmailFilterRow>(&query);
        for cid in concept_ids {
            q = q.bind(cid.to_string());
        }
        if mode == "intersection" {
            q = q.bind(concept_ids.len() as i64);
        }
        q = q.bind(limit).bind(offset);

        let rows = q
            .fetch_all(self.pool)
            .await
            .map_err(|e| DomainError::Internal(format!("Failed to filter emails: {}", e)))?;

        // For each email, get its domain memberships
        let mut results = Vec::new();
        for row in rows {
            let email_id = row.id.clone();
            let domains = sqlx::query_as::<_, DomainMembershipRow>(
                "SELECT cc.concept_id, c.name as concept_name
                 FROM communication_concepts cc
                 INNER JOIN concepts c ON c.id = cc.concept_id
                 WHERE cc.communication_id = ? AND cc.communication_type = 'email'
                 AND cc.status = 'confirmed'",
            )
            .bind(&email_id)
            .fetch_all(self.pool)
            .await
            .map_err(|e| {
                DomainError::Internal(format!("Failed to fetch domain memberships: {}", e))
            })?;

            results.push(EmailWithDomains {
                id: email_id,
                from_address: row.from_address,
                from_name: row.from_name,
                subject: row.subject,
                message_date: row.message_date,
                body_preview: row.body_text.map(|b| {
                    if b.len() > 200 {
                        format!("{}...", &b[..200])
                    } else {
                        b
                    }
                }),
                message_type: row.message_type,
                folder: row.folder,
                domains: domains
                    .into_iter()
                    .map(|d| DomainMembership {
                        concept_id: d.concept_id,
                        concept_name: d.concept_name,
                    })
                    .collect(),
            });
        }

        Ok(results)
    }

    /// Get overlap counts between domain combinations (for Venn diagram rendering)
    pub async fn get_domain_overlap_counts(
        &self,
        domain_ids: &[Uuid],
    ) -> DomainResult<Vec<OverlapCount>> {
        if domain_ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut results = Vec::new();

        // For each domain, get count
        for domain_id in domain_ids {
            let count: (i64,) = sqlx::query_as(
                "SELECT COUNT(DISTINCT communication_id) FROM communication_concepts
                 WHERE concept_id = ? AND communication_type = 'email' AND status = 'confirmed'",
            )
            .bind(domain_id.to_string())
            .fetch_one(self.pool)
            .await
            .map_err(|e| DomainError::Internal(format!("Failed to count domain: {}", e)))?;

            results.push(OverlapCount {
                domain_ids: vec![domain_id.to_string()],
                count: count.0,
            });
        }

        // For pairs, get intersection counts
        for i in 0..domain_ids.len() {
            for j in (i + 1)..domain_ids.len() {
                let count: (i64,) = sqlx::query_as(
                    "SELECT COUNT(*) FROM (
                        SELECT communication_id FROM communication_concepts
                        WHERE concept_id = ? AND communication_type = 'email' AND status = 'confirmed'
                        INTERSECT
                        SELECT communication_id FROM communication_concepts
                        WHERE concept_id = ? AND communication_type = 'email' AND status = 'confirmed'
                    )",
                )
                .bind(domain_ids[i].to_string())
                .bind(domain_ids[j].to_string())
                .fetch_one(self.pool)
                .await
                .map_err(|e| DomainError::Internal(format!("Failed to count overlap: {}", e)))?;

                results.push(OverlapCount {
                    domain_ids: vec![domain_ids[i].to_string(), domain_ids[j].to_string()],
                    count: count.0,
                });
            }
        }

        // For triple if exactly 3 domains
        if domain_ids.len() == 3 {
            let count: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM (
                    SELECT communication_id FROM communication_concepts
                    WHERE concept_id = ? AND communication_type = 'email' AND status = 'confirmed'
                    INTERSECT
                    SELECT communication_id FROM communication_concepts
                    WHERE concept_id = ? AND communication_type = 'email' AND status = 'confirmed'
                    INTERSECT
                    SELECT communication_id FROM communication_concepts
                    WHERE concept_id = ? AND communication_type = 'email' AND status = 'confirmed'
                )",
            )
            .bind(domain_ids[0].to_string())
            .bind(domain_ids[1].to_string())
            .bind(domain_ids[2].to_string())
            .fetch_one(self.pool)
            .await
            .map_err(|e| DomainError::Internal(format!("Failed to count triple overlap: {}", e)))?;

            results.push(OverlapCount {
                domain_ids: domain_ids.iter().map(|d| d.to_string()).collect(),
                count: count.0,
            });
        }

        Ok(results)
    }

    /// Update contact_id for emails from a specific address (used after creating/matching contact)
    pub async fn link_emails_to_contact(
        &self,
        email_address: &str,
        contact_id: Uuid,
    ) -> DomainResult<u64> {
        let result = sqlx::query(
            "UPDATE email_history SET contact_id = ? WHERE LOWER(from_address) = LOWER(?) AND contact_id IS NULL",
        )
        .bind(contact_id.to_string())
        .bind(email_address)
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(format!("Failed to link emails to contact: {}", e)))?;

        Ok(result.rows_affected())
    }

    /// Get all unique senders that have no linked contact
    pub async fn get_unlinked_senders(&self) -> DomainResult<Vec<UnlinkedSender>> {
        let rows = sqlx::query_as::<_, UnlinkedSenderRow>(
            r#"
            SELECT
                from_address as identifier,
                from_name as display_name,
                COUNT(*) as message_count,
                MAX(message_date) as last_message_date
            FROM email_history
            WHERE contact_id IS NULL
            GROUP BY LOWER(from_address)
            ORDER BY message_count DESC
            "#,
        )
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(format!("Failed to fetch unlinked senders: {}", e)))?;

        Ok(rows
            .into_iter()
            .map(|r| UnlinkedSender {
                identifier: r.identifier,
                display_name: r.display_name,
                message_count: r.message_count,
                last_message_date: r.last_message_date,
            })
            .collect())
    }

    /// Search emails by subject, body, or address
    pub async fn search(&self, query: &str, limit: i64) -> DomainResult<Vec<EmailMessage>> {
        let pattern = format!("%{}%", query);
        let rows = sqlx::query_as::<_, EmailMessageRow>(
            "SELECT id, contact_id, from_address, from_name, to_address, cc, bcc,
             subject, body_text, body_html, message_date, message_type, message_id,
             in_reply_to, email_references, read_status, has_attachments, folder,
             imported_at, imported_by, source_server
             FROM email_history
             WHERE subject LIKE ? OR body_text LIKE ? OR from_address LIKE ? OR to_address LIKE ?
             ORDER BY message_date DESC
             LIMIT ?",
        )
        .bind(&pattern)
        .bind(&pattern)
        .bind(&pattern)
        .bind(&pattern)
        .bind(limit)
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(format!("Failed to search emails: {}", e)))?;

        rows.into_iter().map(|row| row.try_into()).collect()
    }
}

#[derive(Debug, Clone)]
pub struct EmailConversation {
    pub from_address: String,
    pub from_name: Option<String>,
    pub contact_id: Option<String>,
    pub message_count: i64,
    pub last_message_date: i64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct EmailWithDomains {
    pub id: String,
    pub from_address: String,
    pub from_name: Option<String>,
    pub subject: String,
    pub message_date: i64,
    pub body_preview: Option<String>,
    pub message_type: i32,
    pub folder: Option<String>,
    pub domains: Vec<DomainMembership>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DomainMembership {
    pub concept_id: String,
    pub concept_name: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct OverlapCount {
    pub domain_ids: Vec<String>,
    pub count: i64,
}

#[derive(Debug, FromRow)]
struct EmailFilterRow {
    id: String,
    from_address: String,
    from_name: Option<String>,
    subject: String,
    message_date: i64,
    body_text: Option<String>,
    message_type: i32,
    folder: Option<String>,
}

#[derive(Debug, FromRow)]
struct DomainMembershipRow {
    concept_id: String,
    concept_name: String,
}

#[derive(Debug, FromRow)]
struct EmailConversationRow {
    from_address: String,
    from_name: Option<String>,
    contact_id: Option<String>,
    message_count: i64,
    last_message_date: i64,
}

#[derive(Debug, FromRow)]
struct EmailMessageRow {
    id: String,
    contact_id: Option<String>,
    from_address: String,
    from_name: Option<String>,
    to_address: String,
    cc: Option<String>,
    bcc: Option<String>,
    subject: String,
    body_text: Option<String>,
    body_html: Option<String>,
    message_date: i64,
    message_type: i32,
    message_id: Option<String>,
    in_reply_to: Option<String>,
    email_references: Option<String>,
    read_status: i32,
    has_attachments: i32,
    folder: Option<String>,
    imported_at: String,
    imported_by: String,
    source_server: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct UnlinkedSender {
    pub identifier: String,
    pub display_name: Option<String>,
    pub message_count: i64,
    pub last_message_date: i64,
}

#[derive(Debug, FromRow)]
struct UnlinkedSenderRow {
    identifier: String,
    display_name: Option<String>,
    message_count: i64,
    last_message_date: i64,
}

impl TryFrom<EmailMessageRow> for EmailMessage {
    type Error = DomainError;

    fn try_from(row: EmailMessageRow) -> Result<Self, Self::Error> {
        let id = Uuid::parse_str(&row.id)
            .map_err(|e| DomainError::Internal(format!("Invalid UUID: {}", e)))?;

        let contact_id = row
            .contact_id
            .map(|id_str| Uuid::parse_str(&id_str))
            .transpose()
            .map_err(|e| DomainError::Internal(format!("Invalid contact_id UUID: {}", e)))?;

        let imported_at = DateTime::parse_from_rfc3339(&row.imported_at)
            .map_err(|e| DomainError::Internal(format!("Invalid imported_at: {}", e)))?
            .with_timezone(&Utc);

        Ok(EmailMessage {
            id,
            contact_id,
            from_address: row.from_address,
            from_name: row.from_name,
            to_address: row.to_address,
            cc: row.cc,
            bcc: row.bcc,
            subject: row.subject,
            body_text: row.body_text,
            body_html: row.body_html,
            message_date: row.message_date,
            message_type: row.message_type,
            message_id: row.message_id,
            in_reply_to: row.in_reply_to,
            email_references: row.email_references,
            read_status: row.read_status,
            has_attachments: row.has_attachments,
            folder: row.folder,
            imported_at,
            imported_by: row.imported_by,
            source_server: row.source_server,
        })
    }
}
