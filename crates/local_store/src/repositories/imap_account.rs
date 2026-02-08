use core_domain::{DomainError, DomainResult};
use crate::db::DbPool;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImapAccount {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub server: String,
    pub port: i32,
    pub username: String,
    pub password: String,
    pub default_folder: String,
    pub max_emails: i32,
    pub last_fetched_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct ImapAccountRepository<'a> {
    pool: &'a DbPool,
}

impl<'a> ImapAccountRepository<'a> {
    pub fn new(pool: &'a DbPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, account: &ImapAccount) -> DomainResult<()> {
        sqlx::query(
            "INSERT INTO imap_accounts (id, user_id, name, server, port, username, password, default_folder, max_emails, last_fetched_at, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(account.id.to_string())
        .bind(account.user_id.to_string())
        .bind(&account.name)
        .bind(&account.server)
        .bind(account.port)
        .bind(&account.username)
        .bind(&account.password)
        .bind(&account.default_folder)
        .bind(account.max_emails)
        .bind(account.last_fetched_at.map(|t| t.to_rfc3339()))
        .bind(account.created_at.to_rfc3339())
        .bind(account.updated_at.to_rfc3339())
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn list_by_user(&self, user_id: Uuid) -> DomainResult<Vec<ImapAccount>> {
        let rows = sqlx::query_as::<_, ImapAccountRow>(
            "SELECT id, user_id, name, server, port, username, password, default_folder, max_emails, last_fetched_at, created_at, updated_at
             FROM imap_accounts WHERE user_id = ? ORDER BY created_at DESC",
        )
        .bind(user_id.to_string())
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into_account()).collect())
    }

    pub async fn get_by_id(&self, id: Uuid) -> DomainResult<ImapAccount> {
        let row = sqlx::query_as::<_, ImapAccountRow>(
            "SELECT id, user_id, name, server, port, username, password, default_folder, max_emails, last_fetched_at, created_at, updated_at
             FROM imap_accounts WHERE id = ?",
        )
        .bind(id.to_string())
        .fetch_optional(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?
        .ok_or_else(|| DomainError::NotFound(format!("ImapAccount {}", id)))?;

        Ok(row.into_account())
    }

    pub async fn update(&self, account: &ImapAccount) -> DomainResult<()> {
        sqlx::query(
            "UPDATE imap_accounts SET name = ?, server = ?, port = ?, username = ?, password = ?, default_folder = ?, max_emails = ?, updated_at = ?
             WHERE id = ?",
        )
        .bind(&account.name)
        .bind(&account.server)
        .bind(account.port)
        .bind(&account.username)
        .bind(&account.password)
        .bind(&account.default_folder)
        .bind(account.max_emails)
        .bind(account.updated_at.to_rfc3339())
        .bind(account.id.to_string())
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn delete(&self, id: Uuid) -> DomainResult<()> {
        sqlx::query("DELETE FROM imap_accounts WHERE id = ?")
            .bind(id.to_string())
            .execute(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn update_last_fetched(&self, id: Uuid, timestamp: DateTime<Utc>) -> DomainResult<()> {
        sqlx::query(
            "UPDATE imap_accounts SET last_fetched_at = ?, updated_at = ? WHERE id = ?",
        )
        .bind(timestamp.to_rfc3339())
        .bind(timestamp.to_rfc3339())
        .bind(id.to_string())
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct ImapAccountRow {
    id: String,
    user_id: String,
    name: String,
    server: String,
    port: i32,
    username: String,
    password: String,
    default_folder: String,
    max_emails: i32,
    last_fetched_at: Option<String>,
    created_at: String,
    updated_at: String,
}

impl ImapAccountRow {
    fn into_account(self) -> ImapAccount {
        ImapAccount {
            id: Uuid::parse_str(&self.id).unwrap(),
            user_id: Uuid::parse_str(&self.user_id).unwrap(),
            name: self.name,
            server: self.server,
            port: self.port,
            username: self.username,
            password: self.password,
            default_folder: self.default_folder,
            max_emails: self.max_emails,
            last_fetched_at: self.last_fetched_at.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|dt| dt.with_timezone(&chrono::Utc))
            }),
            created_at: chrono::DateTime::parse_from_rfc3339(&self.created_at)
                .unwrap()
                .with_timezone(&chrono::Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(&self.updated_at)
                .unwrap()
                .with_timezone(&chrono::Utc),
        }
    }
}
