use core_domain::{CommunicationAttempt, CommunicationStatus, DomainResult, DomainError};
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

pub struct CommunicationRepository<'a> {
    pool: &'a Pool<Sqlite>,
}

impl<'a> CommunicationRepository<'a> {
    pub fn new(pool: &'a Pool<Sqlite>) -> Self {
        Self { pool }
    }

    pub async fn create(&self, attempt: &CommunicationAttempt) -> DomainResult<()> {
        let method_str = serde_json::to_string(&attempt.method).unwrap();
        let status_str = serde_json::to_string(&attempt.status).unwrap();

        sqlx::query(
            "INSERT INTO communication_attempts (id, contact_id, method, subject, message, status, scheduled_at, attempted_at, retry_count, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(attempt.id.to_string())
        .bind(attempt.contact_id.to_string())
        .bind(method_str)
        .bind(&attempt.subject)
        .bind(&attempt.message)
        .bind(status_str)
        .bind(attempt.scheduled_at.map(|t| t.to_rfc3339()))
        .bind(attempt.attempted_at.map(|t| t.to_rfc3339()))
        .bind(attempt.retry_count)
        .bind(attempt.created_at.to_rfc3339())
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn update_status(&self, id: Uuid, status: CommunicationStatus, attempted_at: Option<chrono::DateTime<chrono::Utc>>) -> DomainResult<()> {
        let status_str = serde_json::to_string(&status).unwrap();

        sqlx::query(
            "UPDATE communication_attempts SET status = ?, attempted_at = ? WHERE id = ?"
        )
        .bind(status_str)
        .bind(attempted_at.map(|t| t.to_rfc3339()))
        .bind(id.to_string())
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn update_retry_count(&self, id: Uuid, retry_count: i32) -> DomainResult<()> {
        sqlx::query(
            "UPDATE communication_attempts SET retry_count = ? WHERE id = ?"
        )
        .bind(retry_count)
        .bind(id.to_string())
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn list_pending(&self) -> DomainResult<Vec<CommunicationAttempt>> {
        let rows = sqlx::query_as::<_, CommAttemptRow>(
            "SELECT id, contact_id, method, subject, message, status, scheduled_at, attempted_at, retry_count, created_at
             FROM communication_attempts
             WHERE status LIKE '%Pending%' OR status LIKE '%Retrying%'
             ORDER BY scheduled_at"
        )
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }
}

#[derive(sqlx::FromRow)]
struct CommAttemptRow {
    id: String,
    contact_id: String,
    method: String,
    subject: Option<String>,
    message: String,
    status: String,
    scheduled_at: Option<String>,
    attempted_at: Option<String>,
    retry_count: i32,
    created_at: String,
}

impl From<CommAttemptRow> for CommunicationAttempt {
    fn from(row: CommAttemptRow) -> Self {
        Self {
            id: Uuid::parse_str(&row.id).unwrap(),
            contact_id: Uuid::parse_str(&row.contact_id).unwrap(),
            method: serde_json::from_str(&row.method).unwrap(),
            subject: row.subject,
            message: row.message,
            status: serde_json::from_str(&row.status).unwrap(),
            scheduled_at: row.scheduled_at.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&chrono::Utc))),
            attempted_at: row.attempted_at.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&chrono::Utc))),
            retry_count: row.retry_count,
            created_at: chrono::DateTime::parse_from_rfc3339(&row.created_at).unwrap().with_timezone(&chrono::Utc),
        }
    }
}