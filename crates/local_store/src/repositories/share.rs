use core_domain::{DomainError, DomainResult, ShareInvite};
use crate::db::DbPool;
use uuid::Uuid;

pub struct ShareRepository<'a> {
    pool: &'a DbPool,
}

impl<'a> ShareRepository<'a> {
    pub fn new(pool: &'a DbPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, invite: &ShareInvite) -> DomainResult<()> {
        let entity_type_str = serde_json::to_string(&invite.entity_type).unwrap();
        let permissions_str = serde_json::to_string(&invite.permissions).unwrap();

        sqlx::query(
            "INSERT INTO share_invites (id, entity_type, entity_id, shared_by, shared_with_email, shared_with_user, permissions, accepted, accepted_at, revoked, revoked_at, created_at, expires_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(invite.id.to_string())
        .bind(entity_type_str)
        .bind(invite.entity_id.to_string())
        .bind(invite.shared_by.to_string())
        .bind(&invite.shared_with_email)
        .bind(invite.shared_with_user.map(|id| id.to_string()))
        .bind(permissions_str)
        .bind(invite.accepted as i32)
        .bind(invite.accepted_at.map(|t| t.to_rfc3339()))
        .bind(invite.revoked as i32)
        .bind(invite.revoked_at.map(|t| t.to_rfc3339()))
        .bind(invite.created_at.to_rfc3339())
        .bind(invite.expires_at.map(|t| t.to_rfc3339()))
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn list_by_email(&self, email: &str) -> DomainResult<Vec<ShareInvite>> {
        let rows = sqlx::query_as::<_, ShareInviteRow>(
            "SELECT id, entity_type, entity_id, shared_by, shared_with_email, shared_with_user, permissions, accepted, accepted_at, revoked, revoked_at, created_at, expires_at
             FROM share_invites WHERE shared_with_email = ? ORDER BY created_at DESC"
        )
        .bind(email)
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn get_by_id(&self, id: Uuid) -> DomainResult<ShareInvite> {
        let row = sqlx::query_as::<_, ShareInviteRow>(
            "SELECT id, entity_type, entity_id, shared_by, shared_with_email, shared_with_user, permissions, accepted, accepted_at, revoked, revoked_at, created_at, expires_at
             FROM share_invites WHERE id = ?"
        )
        .bind(id.to_string())
        .fetch_one(self.pool)
        .await
        .map_err(|e| DomainError::NotFound(format!("Share invite not found: {}", e)))?;

        Ok(row.into())
    }

    pub async fn update(&self, invite: &ShareInvite) -> DomainResult<()> {
        let entity_type_str = serde_json::to_string(&invite.entity_type).unwrap();
        let permissions_str = serde_json::to_string(&invite.permissions).unwrap();

        sqlx::query(
            "UPDATE share_invites SET
             entity_type = ?, entity_id = ?, shared_by = ?, shared_with_email = ?,
             shared_with_user = ?, permissions = ?, accepted = ?, accepted_at = ?,
             revoked = ?, revoked_at = ?, expires_at = ?
             WHERE id = ?",
        )
        .bind(entity_type_str)
        .bind(invite.entity_id.to_string())
        .bind(invite.shared_by.to_string())
        .bind(&invite.shared_with_email)
        .bind(invite.shared_with_user.map(|id| id.to_string()))
        .bind(permissions_str)
        .bind(invite.accepted as i32)
        .bind(invite.accepted_at.map(|t| t.to_rfc3339()))
        .bind(invite.revoked as i32)
        .bind(invite.revoked_at.map(|t| t.to_rfc3339()))
        .bind(invite.expires_at.map(|t| t.to_rfc3339()))
        .bind(invite.id.to_string())
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn list_by_sharer(&self, user_id: Uuid) -> DomainResult<Vec<ShareInvite>> {
        let rows = sqlx::query_as::<_, ShareInviteRow>(
            "SELECT id, entity_type, entity_id, shared_by, shared_with_email, shared_with_user, permissions, accepted, accepted_at, revoked, revoked_at, created_at, expires_at
             FROM share_invites WHERE shared_by = ? ORDER BY created_at DESC"
        )
        .bind(user_id.to_string())
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_by_recipient(&self, email: &str) -> DomainResult<Vec<ShareInvite>> {
        self.list_by_email(email).await
    }
}

#[derive(sqlx::FromRow)]
struct ShareInviteRow {
    id: String,
    entity_type: String,
    entity_id: String,
    shared_by: String,
    shared_with_email: String,
    shared_with_user: Option<String>,
    permissions: String,
    accepted: i32,
    accepted_at: Option<String>,
    revoked: i32,
    revoked_at: Option<String>,
    created_at: String,
    expires_at: Option<String>,
}

impl From<ShareInviteRow> for ShareInvite {
    fn from(row: ShareInviteRow) -> Self {
        Self {
            id: Uuid::parse_str(&row.id).unwrap(),
            entity_type: serde_json::from_str(&row.entity_type).unwrap(),
            entity_id: Uuid::parse_str(&row.entity_id).unwrap(),
            shared_by: Uuid::parse_str(&row.shared_by).unwrap(),
            shared_with_email: row.shared_with_email,
            shared_with_user: row.shared_with_user.and_then(|s| Uuid::parse_str(&s).ok()),
            permissions: serde_json::from_str(&row.permissions).unwrap(),
            accepted: row.accepted != 0,
            accepted_at: row.accepted_at.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|dt| dt.with_timezone(&chrono::Utc))
            }),
            revoked: row.revoked != 0,
            revoked_at: row.revoked_at.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|dt| dt.with_timezone(&chrono::Utc))
            }),
            created_at: chrono::DateTime::parse_from_rfc3339(&row.created_at)
                .unwrap()
                .with_timezone(&chrono::Utc),
            expires_at: row.expires_at.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|dt| dt.with_timezone(&chrono::Utc))
            }),
        }
    }
}
