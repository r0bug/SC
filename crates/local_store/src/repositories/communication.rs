use core_domain::{Communication, CommunicationAttempt, CommunicationDirection, CommunicationType, CommunicationHistoryStatus, CommunicationStatus, DomainError, DomainResult};
use sqlx::{Pool, Sqlite, FromRow};
use uuid::Uuid;
use chrono::{DateTime, Utc};

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

    pub async fn update_status(
        &self,
        id: Uuid,
        status: CommunicationStatus,
        attempted_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> DomainResult<()> {
        let status_str = serde_json::to_string(&status).unwrap();

        sqlx::query("UPDATE communication_attempts SET status = ?, attempted_at = ? WHERE id = ?")
            .bind(status_str)
            .bind(attempted_at.map(|t| t.to_rfc3339()))
            .bind(id.to_string())
            .execute(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn update_retry_count(&self, id: Uuid, retry_count: i32) -> DomainResult<()> {
        sqlx::query("UPDATE communication_attempts SET retry_count = ? WHERE id = ?")
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

    // Historical Communication methods
    pub async fn create_communication(&self, communication: &Communication) -> DomainResult<()> {
        let communication_type = match &communication.communication_type {
            CommunicationType::Sms => "Sms",
            CommunicationType::Call => "Call",
            CommunicationType::Email => "Email",
            CommunicationType::VideoCall => "VideoCall",
            CommunicationType::VoiceMail => "VoiceMail",
        };

        let direction = match &communication.direction {
            CommunicationDirection::Inbound => "Inbound",
            CommunicationDirection::Outbound => "Outbound",
            CommunicationDirection::Missed => "Missed",
        };

        let status = match &communication.status {
            CommunicationHistoryStatus::Completed => "Completed",
            CommunicationHistoryStatus::Failed => "Failed",
            CommunicationHistoryStatus::Blocked => "Blocked",
            CommunicationHistoryStatus::Rejected => "Rejected",
        };

        sqlx::query(
            "INSERT INTO communications (id, contact_id, communication_type, direction, timestamp, content, duration_seconds, phone_number, thread_id, status, metadata, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(communication.id.to_string())
        .bind(communication.contact_id.to_string())
        .bind(communication_type)
        .bind(direction)
        .bind(communication.timestamp.to_rfc3339())
        .bind(&communication.content)
        .bind(communication.duration_seconds)
        .bind(&communication.phone_number)
        .bind(&communication.thread_id)
        .bind(status)
        .bind(serde_json::to_string(&communication.metadata).unwrap())
        .bind(communication.created_at.to_rfc3339())
        .bind(communication.updated_at.to_rfc3339())
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn get_communications_by_contact(&self, contact_id: Uuid) -> DomainResult<Vec<Communication>> {
        let rows = sqlx::query_as::<_, CommunicationRow>(
            "SELECT id, contact_id, communication_type, direction, timestamp, content, duration_seconds, phone_number, thread_id, status, metadata, created_at, updated_at
             FROM communications WHERE contact_id = ? ORDER BY timestamp DESC"
        )
        .bind(contact_id.to_string())
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        rows.into_iter().map(|row| row.try_into()).collect()
    }

    pub async fn get_communications_by_phone(&self, phone_number: &str) -> DomainResult<Vec<Communication>> {
        let rows = sqlx::query_as::<_, CommunicationRow>(
            "SELECT id, contact_id, communication_type, direction, timestamp, content, duration_seconds, phone_number, thread_id, status, metadata, created_at, updated_at
             FROM communications WHERE phone_number = ? ORDER BY timestamp DESC"
        )
        .bind(phone_number)
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        rows.into_iter().map(|row| row.try_into()).collect()
    }

    pub async fn get_communications_by_thread(&self, thread_id: &str) -> DomainResult<Vec<Communication>> {
        let rows = sqlx::query_as::<_, CommunicationRow>(
            "SELECT id, contact_id, communication_type, direction, timestamp, content, duration_seconds, phone_number, thread_id, status, metadata, created_at, updated_at
             FROM communications WHERE thread_id = ? ORDER BY timestamp ASC"
        )
        .bind(thread_id)
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        rows.into_iter().map(|row| row.try_into()).collect()
    }

    pub async fn count_communications_by_contact(&self, contact_id: Uuid) -> DomainResult<i64> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM communications WHERE contact_id = ?"
        )
        .bind(contact_id.to_string())
        .fetch_one(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(count.0)
    }

    pub async fn list_all(&self, limit: i64, offset: i64) -> DomainResult<Vec<Communication>> {
        let rows = sqlx::query_as::<_, CommunicationRow>(
            "SELECT id, contact_id, communication_type, direction, timestamp, content, duration_seconds, phone_number, thread_id, status, metadata, created_at, updated_at
             FROM communications ORDER BY timestamp DESC LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        rows.into_iter().map(|row| row.try_into()).collect()
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
            scheduled_at: row.scheduled_at.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|dt| dt.with_timezone(&chrono::Utc))
            }),
            attempted_at: row.attempted_at.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|dt| dt.with_timezone(&chrono::Utc))
            }),
            retry_count: row.retry_count,
            created_at: chrono::DateTime::parse_from_rfc3339(&row.created_at)
                .unwrap()
                .with_timezone(&chrono::Utc),
        }
    }
}

#[derive(Debug, FromRow)]
struct CommunicationRow {
    id: String,
    contact_id: String,
    communication_type: String,
    direction: String,
    timestamp: String,
    content: Option<String>,
    duration_seconds: Option<i32>,
    phone_number: Option<String>,
    thread_id: Option<String>,
    status: String,
    metadata: String,
    created_at: String,
    updated_at: String,
}

impl TryFrom<CommunicationRow> for Communication {
    type Error = DomainError;

    fn try_from(row: CommunicationRow) -> Result<Self, Self::Error> {
        let id = Uuid::parse_str(&row.id)
            .map_err(|e| DomainError::Internal(format!("Invalid UUID: {}", e)))?;

        let contact_id = Uuid::parse_str(&row.contact_id)
            .map_err(|e| DomainError::Internal(format!("Invalid contact_id UUID: {}", e)))?;

        let communication_type = match row.communication_type.as_str() {
            "Sms" => CommunicationType::Sms,
            "Call" => CommunicationType::Call,
            "Email" => CommunicationType::Email,
            "VideoCall" => CommunicationType::VideoCall,
            "VoiceMail" => CommunicationType::VoiceMail,
            _ => return Err(DomainError::Internal(format!("Invalid communication type: {}", row.communication_type))),
        };

        let direction = match row.direction.as_str() {
            "Inbound" => CommunicationDirection::Inbound,
            "Outbound" => CommunicationDirection::Outbound,
            "Missed" => CommunicationDirection::Missed,
            _ => return Err(DomainError::Internal(format!("Invalid direction: {}", row.direction))),
        };

        let status = match row.status.as_str() {
            "Completed" => CommunicationHistoryStatus::Completed,
            "Failed" => CommunicationHistoryStatus::Failed,
            "Blocked" => CommunicationHistoryStatus::Blocked,
            "Rejected" => CommunicationHistoryStatus::Rejected,
            _ => return Err(DomainError::Internal(format!("Invalid status: {}", row.status))),
        };

        let timestamp = DateTime::parse_from_rfc3339(&row.timestamp)
            .map_err(|e| DomainError::Internal(format!("Invalid timestamp: {}", e)))?
            .with_timezone(&Utc);

        let created_at = DateTime::parse_from_rfc3339(&row.created_at)
            .map_err(|e| DomainError::Internal(format!("Invalid created_at: {}", e)))?
            .with_timezone(&Utc);

        let updated_at = DateTime::parse_from_rfc3339(&row.updated_at)
            .map_err(|e| DomainError::Internal(format!("Invalid updated_at: {}", e)))?
            .with_timezone(&Utc);

        let metadata = serde_json::from_str(&row.metadata)
            .map_err(|e| DomainError::Internal(format!("Invalid metadata JSON: {}", e)))?;

        Ok(Communication {
            id,
            contact_id,
            communication_type,
            direction,
            timestamp,
            content: row.content,
            duration_seconds: row.duration_seconds,
            phone_number: row.phone_number,
            thread_id: row.thread_id,
            status,
            metadata,
            created_at,
            updated_at,
        })
    }
}
