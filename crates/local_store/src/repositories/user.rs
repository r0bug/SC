use core_domain::{DomainError, DomainResult, User};
use crate::db::DbPool;
use uuid::Uuid;

pub struct UserRepository<'a> {
    pool: &'a DbPool,
}

impl<'a> UserRepository<'a> {
    pub fn new(pool: &'a DbPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, user: &User) -> DomainResult<()> {
        sqlx::query(
            "INSERT INTO users (id, email, name, password_hash, api_token, email_verified, active, preferences, created_at, updated_at, last_login_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(user.id.to_string())
        .bind(&user.email)
        .bind(&user.name)
        .bind(&user.password_hash)
        .bind(&user.api_token)
        .bind(user.email_verified as i32)
        .bind(user.active as i32)
        .bind(serde_json::to_string(&user.preferences).unwrap())
        .bind(user.created_at.to_rfc3339())
        .bind(user.updated_at.to_rfc3339())
        .bind(user.last_login_at.map(|t| t.to_rfc3339()))
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn get_by_id(&self, id: Uuid) -> DomainResult<User> {
        let row = sqlx::query_as::<_, UserRow>(
            "SELECT id, email, name, password_hash, api_token, email_verified, active, preferences, created_at, updated_at, last_login_at
             FROM users WHERE id = ?"
        )
        .bind(id.to_string())
        .fetch_optional(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?
        .ok_or_else(|| DomainError::NotFound(format!("User {}", id)))?;

        Ok(row.into())
    }

    pub async fn get_by_email(&self, email: &str) -> DomainResult<User> {
        let row = sqlx::query_as::<_, UserRow>(
            "SELECT id, email, name, password_hash, api_token, email_verified, active, preferences, created_at, updated_at, last_login_at
             FROM users WHERE email = ?"
        )
        .bind(email)
        .fetch_optional(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?
        .ok_or_else(|| DomainError::NotFound(format!("User with email {}", email)))?;

        Ok(row.into())
    }

    pub async fn get_by_api_token(&self, token: &str) -> DomainResult<User> {
        let row = sqlx::query_as::<_, UserRow>(
            "SELECT id, email, name, password_hash, api_token, email_verified, active, preferences, created_at, updated_at, last_login_at
             FROM users WHERE api_token = ? AND active = 1"
        )
        .bind(token)
        .fetch_optional(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?
        .ok_or_else(|| DomainError::NotFound("User with token".to_string()))?;

        Ok(row.into())
    }

    pub async fn update(&self, user: &User) -> DomainResult<()> {
        sqlx::query(
            "UPDATE users SET email = ?, name = ?, password_hash = ?, api_token = ?, email_verified = ?, active = ?, preferences = ?, updated_at = ?, last_login_at = ?
             WHERE id = ?"
        )
        .bind(&user.email)
        .bind(&user.name)
        .bind(&user.password_hash)
        .bind(&user.api_token)
        .bind(user.email_verified as i32)
        .bind(user.active as i32)
        .bind(serde_json::to_string(&user.preferences).unwrap())
        .bind(user.updated_at.to_rfc3339())
        .bind(user.last_login_at.map(|t| t.to_rfc3339()))
        .bind(user.id.to_string())
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn delete(&self, id: Uuid) -> DomainResult<()> {
        sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(id.to_string())
            .execute(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct UserRow {
    id: String,
    email: String,
    name: String,
    password_hash: String,
    api_token: Option<String>,
    email_verified: i32,
    active: i32,
    preferences: String,
    created_at: String,
    updated_at: String,
    last_login_at: Option<String>,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        Self {
            id: Uuid::parse_str(&row.id).unwrap(),
            email: row.email,
            name: row.name,
            password_hash: row.password_hash,
            api_token: row.api_token,
            email_verified: row.email_verified != 0,
            active: row.active != 0,
            preferences: serde_json::from_str(&row.preferences).unwrap_or(serde_json::json!({})),
            created_at: chrono::DateTime::parse_from_rfc3339(&row.created_at)
                .unwrap()
                .with_timezone(&chrono::Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(&row.updated_at)
                .unwrap()
                .with_timezone(&chrono::Utc),
            last_login_at: row.last_login_at.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|dt| dt.with_timezone(&chrono::Utc))
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup_test_db() -> DbPool {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query(crate::migrations::SCHEMA)
            .execute(&pool)
            .await
            .unwrap();

        pool
    }

    #[tokio::test]
    async fn test_create_and_get_user() {
        let pool = setup_test_db().await;
        let repo = UserRepository::new(&pool);

        let user = User {
            id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            name: "Test User".to_string(),
            password_hash: "hashed_password".to_string(),
            api_token: Some("test_token".to_string()),
            email_verified: true,
            active: true,
            preferences: serde_json::json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login_at: None,
        };

        repo.create(&user).await.unwrap();

        let retrieved = repo.get_by_id(user.id).await.unwrap();
        assert_eq!(retrieved.email, user.email);
        assert_eq!(retrieved.name, user.name);
    }

    #[tokio::test]
    async fn test_get_by_email() {
        let pool = setup_test_db().await;
        let repo = UserRepository::new(&pool);

        let user = User {
            id: Uuid::new_v4(),
            email: "email@example.com".to_string(),
            name: "Email User".to_string(),
            password_hash: "hashed".to_string(),
            api_token: None,
            email_verified: false,
            active: true,
            preferences: serde_json::json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login_at: None,
        };

        repo.create(&user).await.unwrap();

        let retrieved = repo.get_by_email("email@example.com").await.unwrap();
        assert_eq!(retrieved.id, user.id);
    }
}
