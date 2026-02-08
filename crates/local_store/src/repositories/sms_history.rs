use crate::db::DbPool;
use chrono::{DateTime, Utc};
use core_domain::{DomainError, DomainResult};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SmsMessage {
    pub id: Uuid,
    pub contact_id: Option<Uuid>,
    pub phone_number: String,
    pub contact_name: Option<String>,
    pub message_date: i64, // Unix timestamp in milliseconds
    pub message_type: i32, // 1=received, 2=sent, 3=draft, 4=outbox, 5=failed, 6=queued
    pub subject: Option<String>,
    pub body: String,
    pub readable_date: String,
    pub thread_id: Option<i32>,
    pub read_status: i32, // 0=unread, 1=read
    pub subscription_id: Option<String>,
    pub imported_at: DateTime<Utc>,
    pub imported_by: String,
    pub source_file: Option<String>,
}

pub struct SmsHistoryRepository<'a> {
    pool: &'a DbPool,
}

impl<'a> SmsHistoryRepository<'a> {
    pub fn new(pool: &'a DbPool) -> Self {
        Self { pool }
    }

    /// Create a new SMS message record
    pub async fn create(&self, sms: &SmsMessage) -> DomainResult<()> {
        sqlx::query(
            "INSERT INTO sms_history (id, contact_id, phone_number, contact_name, message_date,
             message_type, subject, body, readable_date, thread_id, read_status, subscription_id,
             imported_at, imported_by, source_file)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(sms.id.to_string())
        .bind(sms.contact_id.map(|id| id.to_string()))
        .bind(&sms.phone_number)
        .bind(&sms.contact_name)
        .bind(sms.message_date)
        .bind(sms.message_type)
        .bind(&sms.subject)
        .bind(&sms.body)
        .bind(&sms.readable_date)
        .bind(sms.thread_id)
        .bind(sms.read_status)
        .bind(&sms.subscription_id)
        .bind(sms.imported_at.to_rfc3339())
        .bind(&sms.imported_by)
        .bind(&sms.source_file)
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(format!("Failed to create SMS message: {}", e)))?;

        Ok(())
    }

    /// Batch insert multiple SMS messages (more efficient for imports)
    pub async fn batch_create(&self, messages: &[SmsMessage]) -> DomainResult<usize> {
        let mut tx =
            self.pool.begin().await.map_err(|e| {
                DomainError::Internal(format!("Failed to start transaction: {}", e))
            })?;

        let mut count = 0;
        for sms in messages {
            sqlx::query(
                "INSERT INTO sms_history (id, contact_id, phone_number, contact_name, message_date,
                 message_type, subject, body, readable_date, thread_id, read_status, subscription_id,
                 imported_at, imported_by, source_file)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(sms.id.to_string())
            .bind(sms.contact_id.map(|id| id.to_string()))
            .bind(&sms.phone_number)
            .bind(&sms.contact_name)
            .bind(sms.message_date)
            .bind(sms.message_type)
            .bind(&sms.subject)
            .bind(&sms.body)
            .bind(&sms.readable_date)
            .bind(sms.thread_id)
            .bind(sms.read_status)
            .bind(&sms.subscription_id)
            .bind(sms.imported_at.to_rfc3339())
            .bind(&sms.imported_by)
            .bind(&sms.source_file)
            .execute(&mut *tx)
            .await
            .map_err(|e| DomainError::Internal(format!("Failed to insert SMS message: {}", e)))?;

            count += 1;
        }

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(format!("Failed to commit transaction: {}", e)))?;

        Ok(count)
    }

    /// List all SMS messages (for bulk operations like retroactive scanning)
    pub async fn list_all(&self) -> DomainResult<Vec<SmsMessage>> {
        let rows = sqlx::query_as::<_, SmsMessageRow>(
            "SELECT id, contact_id, phone_number, contact_name, message_date, message_type,
             subject, body, readable_date, thread_id, read_status, subscription_id,
             imported_at, imported_by, source_file
             FROM sms_history ORDER BY message_date DESC",
        )
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(format!("Failed to list all SMS: {}", e)))?;

        rows.into_iter().map(|row| row.try_into()).collect()
    }

    /// Get all SMS messages for a contact
    pub async fn get_by_contact(&self, contact_id: Uuid) -> DomainResult<Vec<SmsMessage>> {
        let rows = sqlx::query_as::<_, SmsMessageRow>(
            "SELECT id, contact_id, phone_number, contact_name, message_date, message_type,
             subject, body, readable_date, thread_id, read_status, subscription_id,
             imported_at, imported_by, source_file
             FROM sms_history WHERE contact_id = ? ORDER BY message_date DESC",
        )
        .bind(contact_id.to_string())
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(format!("Failed to fetch SMS messages: {}", e)))?;

        rows.into_iter().map(|row| row.try_into()).collect()
    }

    /// Get all SMS messages for a phone number (even if not linked to contact)
    pub async fn get_by_phone(&self, phone_number: &str) -> DomainResult<Vec<SmsMessage>> {
        let rows = sqlx::query_as::<_, SmsMessageRow>(
            "SELECT id, contact_id, phone_number, contact_name, message_date, message_type,
             subject, body, readable_date, thread_id, read_status, subscription_id,
             imported_at, imported_by, source_file
             FROM sms_history WHERE phone_number = ? ORDER BY message_date DESC",
        )
        .bind(phone_number)
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(format!("Failed to fetch SMS messages: {}", e)))?;

        rows.into_iter().map(|row| row.try_into()).collect()
    }

    /// Get all SMS messages in a thread
    pub async fn get_by_thread(&self, thread_id: i32) -> DomainResult<Vec<SmsMessage>> {
        let rows = sqlx::query_as::<_, SmsMessageRow>(
            "SELECT id, contact_id, phone_number, contact_name, message_date, message_type,
             subject, body, readable_date, thread_id, read_status, subscription_id,
             imported_at, imported_by, source_file
             FROM sms_history WHERE thread_id = ? ORDER BY message_date ASC",
        )
        .bind(thread_id)
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(format!("Failed to fetch SMS messages: {}", e)))?;

        rows.into_iter().map(|row| row.try_into()).collect()
    }

    /// Count SMS messages for a contact
    pub async fn count_by_contact(&self, contact_id: Uuid) -> DomainResult<i64> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sms_history WHERE contact_id = ?")
            .bind(contact_id.to_string())
            .fetch_one(self.pool)
            .await
            .map_err(|e| DomainError::Internal(format!("Failed to count SMS messages: {}", e)))?;

        Ok(count.0)
    }

    /// Get all unique phone numbers that have no linked contact
    pub async fn get_unlinked_senders(&self) -> DomainResult<Vec<UnlinkedSmsSender>> {
        let rows = sqlx::query_as::<_, UnlinkedSmsSenderRow>(
            r#"
            SELECT
                phone_number,
                contact_name,
                COUNT(*) as message_count,
                MAX(message_date) as last_message_date
            FROM sms_history
            WHERE contact_id IS NULL
            GROUP BY phone_number
            ORDER BY message_count DESC
            "#,
        )
        .fetch_all(self.pool)
        .await
        .map_err(|e| {
            DomainError::Internal(format!("Failed to fetch unlinked SMS senders: {}", e))
        })?;

        Ok(rows
            .into_iter()
            .map(|r| UnlinkedSmsSender {
                phone_number: r.phone_number,
                contact_name: r.contact_name,
                message_count: r.message_count,
                last_message_date: r.last_message_date,
            })
            .collect())
    }

    /// Update contact_id for messages with a specific phone number (used after creating contact)
    pub async fn link_messages_to_contact(
        &self,
        phone_number: &str,
        contact_id: Uuid,
    ) -> DomainResult<u64> {
        let result = sqlx::query(
            "UPDATE sms_history SET contact_id = ? WHERE phone_number = ? AND contact_id IS NULL",
        )
        .bind(contact_id.to_string())
        .bind(phone_number)
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(format!("Failed to link messages to contact: {}", e)))?;

        Ok(result.rows_affected())
    }
}

#[derive(Debug, FromRow)]
struct SmsMessageRow {
    id: String,
    contact_id: Option<String>,
    phone_number: String,
    contact_name: Option<String>,
    message_date: i64,
    message_type: i32,
    subject: Option<String>,
    body: String,
    readable_date: String,
    thread_id: Option<i32>,
    read_status: i32,
    subscription_id: Option<String>,
    imported_at: String,
    imported_by: String,
    source_file: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct UnlinkedSmsSender {
    pub phone_number: String,
    pub contact_name: Option<String>,
    pub message_count: i64,
    pub last_message_date: i64,
}

#[derive(Debug, FromRow)]
struct UnlinkedSmsSenderRow {
    phone_number: String,
    contact_name: Option<String>,
    message_count: i64,
    last_message_date: i64,
}

impl TryFrom<SmsMessageRow> for SmsMessage {
    type Error = DomainError;

    fn try_from(row: SmsMessageRow) -> Result<Self, Self::Error> {
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

        Ok(SmsMessage {
            id,
            contact_id,
            phone_number: row.phone_number,
            contact_name: row.contact_name,
            message_date: row.message_date,
            message_type: row.message_type,
            subject: row.subject,
            body: row.body,
            readable_date: row.readable_date,
            thread_id: row.thread_id,
            read_status: row.read_status,
            subscription_id: row.subscription_id,
            imported_at,
            imported_by: row.imported_by,
            source_file: row.source_file,
        })
    }
}
