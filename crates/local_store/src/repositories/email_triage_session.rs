use chrono::{DateTime, Utc};
use core_domain::{DomainError, DomainResult};
use sqlx::FromRow;
use crate::db::DbPool;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct EmailTriageSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub server: String,
    pub folder: String,
    pub status: String,
    pub total_fetched: i64,
    pub total_analyzed: i64,
    pub total_categorized: i64,
    pub current_block: i64,
    pub block_size: i64,
    pub discovered_domains: String, // JSON array
    pub approved_domains: String,   // JSON array
    pub denied_domains: String,     // JSON array
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

pub struct EmailTriageSessionRepository<'a> {
    pool: &'a DbPool,
}

impl<'a> EmailTriageSessionRepository<'a> {
    pub fn new(pool: &'a DbPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, session: &EmailTriageSession) -> DomainResult<()> {
        sqlx::query(
            "INSERT INTO email_triage_sessions (id, user_id, server, folder, status,
             total_fetched, total_analyzed, total_categorized, current_block, block_size,
             discovered_domains, approved_domains, denied_domains, error_message,
             created_at, updated_at, completed_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(session.id.to_string())
        .bind(session.user_id.to_string())
        .bind(&session.server)
        .bind(&session.folder)
        .bind(&session.status)
        .bind(session.total_fetched)
        .bind(session.total_analyzed)
        .bind(session.total_categorized)
        .bind(session.current_block)
        .bind(session.block_size)
        .bind(&session.discovered_domains)
        .bind(&session.approved_domains)
        .bind(&session.denied_domains)
        .bind(&session.error_message)
        .bind(session.created_at.to_rfc3339())
        .bind(session.updated_at.to_rfc3339())
        .bind(session.completed_at.map(|dt| dt.to_rfc3339()))
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(format!("Failed to create triage session: {}", e)))?;

        Ok(())
    }

    pub async fn get_by_id(&self, id: Uuid) -> DomainResult<EmailTriageSession> {
        let row = sqlx::query_as::<_, TriageSessionRow>(
            "SELECT id, user_id, server, folder, status, total_fetched, total_analyzed,
             total_categorized, current_block, block_size, discovered_domains,
             approved_domains, denied_domains, error_message, created_at, updated_at, completed_at
             FROM email_triage_sessions WHERE id = ?",
        )
        .bind(id.to_string())
        .fetch_optional(self.pool)
        .await
        .map_err(|e| DomainError::Internal(format!("Failed to fetch triage session: {}", e)))?
        .ok_or_else(|| DomainError::NotFound(format!("Triage session {}", id)))?;

        row.try_into()
    }

    pub async fn list_by_user(&self, user_id: Uuid) -> DomainResult<Vec<EmailTriageSession>> {
        let rows = sqlx::query_as::<_, TriageSessionRow>(
            "SELECT id, user_id, server, folder, status, total_fetched, total_analyzed,
             total_categorized, current_block, block_size, discovered_domains,
             approved_domains, denied_domains, error_message, created_at, updated_at, completed_at
             FROM email_triage_sessions WHERE user_id = ? ORDER BY created_at DESC",
        )
        .bind(user_id.to_string())
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(format!("Failed to list triage sessions: {}", e)))?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }

    pub async fn update_status(&self, id: Uuid, status: &str, error_message: Option<&str>) -> DomainResult<()> {
        let now = Utc::now().to_rfc3339();
        let completed_at = if status == "completed" || status == "failed" {
            Some(now.clone())
        } else {
            None
        };

        sqlx::query(
            "UPDATE email_triage_sessions SET status = ?, error_message = ?,
             updated_at = ?, completed_at = COALESCE(?, completed_at) WHERE id = ?",
        )
        .bind(status)
        .bind(error_message)
        .bind(&now)
        .bind(completed_at)
        .bind(id.to_string())
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(format!("Failed to update triage status: {}", e)))?;

        Ok(())
    }

    pub async fn update_progress(
        &self,
        id: Uuid,
        total_fetched: i64,
        total_analyzed: i64,
        total_categorized: i64,
        current_block: i64,
    ) -> DomainResult<()> {
        sqlx::query(
            "UPDATE email_triage_sessions SET total_fetched = ?, total_analyzed = ?,
             total_categorized = ?, current_block = ?, updated_at = ? WHERE id = ?",
        )
        .bind(total_fetched)
        .bind(total_analyzed)
        .bind(total_categorized)
        .bind(current_block)
        .bind(Utc::now().to_rfc3339())
        .bind(id.to_string())
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(format!("Failed to update triage progress: {}", e)))?;

        Ok(())
    }

    pub async fn update_domains(
        &self,
        id: Uuid,
        discovered: &str,
        approved: &str,
        denied: &str,
    ) -> DomainResult<()> {
        sqlx::query(
            "UPDATE email_triage_sessions SET discovered_domains = ?, approved_domains = ?,
             denied_domains = ?, updated_at = ? WHERE id = ?",
        )
        .bind(discovered)
        .bind(approved)
        .bind(denied)
        .bind(Utc::now().to_rfc3339())
        .bind(id.to_string())
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(format!("Failed to update triage domains: {}", e)))?;

        Ok(())
    }
}

#[derive(Debug, FromRow)]
struct TriageSessionRow {
    id: String,
    user_id: String,
    server: String,
    folder: String,
    status: String,
    total_fetched: i64,
    total_analyzed: i64,
    total_categorized: i64,
    current_block: i64,
    block_size: i64,
    discovered_domains: String,
    approved_domains: String,
    denied_domains: String,
    error_message: Option<String>,
    created_at: String,
    updated_at: String,
    completed_at: Option<String>,
}

impl TryFrom<TriageSessionRow> for EmailTriageSession {
    type Error = DomainError;

    fn try_from(row: TriageSessionRow) -> Result<Self, Self::Error> {
        let id = Uuid::parse_str(&row.id)
            .map_err(|e| DomainError::Internal(format!("Invalid UUID: {}", e)))?;
        let user_id = Uuid::parse_str(&row.user_id)
            .map_err(|e| DomainError::Internal(format!("Invalid user_id UUID: {}", e)))?;
        let created_at = DateTime::parse_from_rfc3339(&row.created_at)
            .map_err(|e| DomainError::Internal(format!("Invalid created_at: {}", e)))?
            .with_timezone(&Utc);
        let updated_at = DateTime::parse_from_rfc3339(&row.updated_at)
            .map_err(|e| DomainError::Internal(format!("Invalid updated_at: {}", e)))?
            .with_timezone(&Utc);
        let completed_at = row
            .completed_at
            .map(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&Utc))
                    .map_err(|e| DomainError::Internal(format!("Invalid completed_at: {}", e)))
            })
            .transpose()?;

        Ok(EmailTriageSession {
            id,
            user_id,
            server: row.server,
            folder: row.folder,
            status: row.status,
            total_fetched: row.total_fetched,
            total_analyzed: row.total_analyzed,
            total_categorized: row.total_categorized,
            current_block: row.current_block,
            block_size: row.block_size,
            discovered_domains: row.discovered_domains,
            approved_domains: row.approved_domains,
            denied_domains: row.denied_domains,
            error_message: row.error_message,
            created_at,
            updated_at,
            completed_at,
        })
    }
}
