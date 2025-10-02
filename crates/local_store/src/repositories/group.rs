use core_domain::{Group, DomainResult, DomainError};
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

pub struct GroupRepository<'a> {
    pool: &'a Pool<Sqlite>,
}

impl<'a> GroupRepository<'a> {
    pub fn new(pool: &'a Pool<Sqlite>) -> Self {
        Self { pool }
    }

    pub async fn create(&self, group: &Group) -> DomainResult<()> {
        let mut tx = self.pool.begin().await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        sqlx::query(
            "INSERT INTO groups (id, name, description, created_at, updated_at, created_by)
             VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(group.id.to_string())
        .bind(&group.name)
        .bind(&group.description)
        .bind(group.created_at.to_rfc3339())
        .bind(group.updated_at.to_rfc3339())
        .bind(group.created_by.to_string())
        .execute(&mut *tx)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        for contact_id in &group.contact_ids {
            sqlx::query(
                "INSERT INTO group_members (group_id, contact_id) VALUES (?, ?)"
            )
            .bind(group.id.to_string())
            .bind(contact_id.to_string())
            .execute(&mut *tx)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        }

        tx.commit().await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn get_by_id(&self, id: Uuid) -> DomainResult<Group> {
        let row = sqlx::query_as::<_, GroupRow>(
            "SELECT id, name, description, created_at, updated_at, created_by
             FROM groups WHERE id = ?"
        )
        .bind(id.to_string())
        .fetch_optional(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?
        .ok_or_else(|| DomainError::NotFound(format!("Group {}", id)))?;

        let contact_ids = sqlx::query_scalar::<_, String>(
            "SELECT contact_id FROM group_members WHERE group_id = ?"
        )
        .bind(id.to_string())
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?
        .into_iter()
        .filter_map(|s| Uuid::parse_str(&s).ok())
        .collect();

        Ok(row.into_group(contact_ids))
    }

    pub async fn list(&self, limit: i64, offset: i64) -> DomainResult<Vec<Group>> {
        let rows = sqlx::query_as::<_, GroupRow>(
            "SELECT id, name, description, created_at, updated_at, created_by
             FROM groups ORDER BY name LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        let mut groups = Vec::new();
        for row in rows {
            let contact_ids = sqlx::query_scalar::<_, String>(
                "SELECT contact_id FROM group_members WHERE group_id = ?"
            )
            .bind(row.id.clone())
            .fetch_all(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?
            .into_iter()
            .filter_map(|s| Uuid::parse_str(&s).ok())
            .collect();

            groups.push(row.into_group(contact_ids));
        }

        Ok(groups)
    }

    pub async fn update(&self, group: &Group) -> DomainResult<()> {
        let mut tx = self.pool.begin().await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        sqlx::query(
            "UPDATE groups SET name = ?, description = ?, updated_at = ?
             WHERE id = ?"
        )
        .bind(&group.name)
        .bind(&group.description)
        .bind(group.updated_at.to_rfc3339())
        .bind(group.id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        sqlx::query("DELETE FROM group_members WHERE group_id = ?")
            .bind(group.id.to_string())
            .execute(&mut *tx)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        for contact_id in &group.contact_ids {
            sqlx::query(
                "INSERT INTO group_members (group_id, contact_id) VALUES (?, ?)"
            )
            .bind(group.id.to_string())
            .bind(contact_id.to_string())
            .execute(&mut *tx)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        }

        tx.commit().await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn delete(&self, id: Uuid) -> DomainResult<()> {
        sqlx::query("DELETE FROM groups WHERE id = ?")
            .bind(id.to_string())
            .execute(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn add_member(&self, group_id: Uuid, contact_id: Uuid) -> DomainResult<()> {
        sqlx::query(
            "INSERT OR IGNORE INTO group_members (group_id, contact_id) VALUES (?, ?)"
        )
        .bind(group_id.to_string())
        .bind(contact_id.to_string())
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn remove_member(&self, group_id: Uuid, contact_id: Uuid) -> DomainResult<()> {
        sqlx::query(
            "DELETE FROM group_members WHERE group_id = ? AND contact_id = ?"
        )
        .bind(group_id.to_string())
        .bind(contact_id.to_string())
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn list_by_creator(&self, creator_id: Uuid) -> DomainResult<Vec<Group>> {
        let rows = sqlx::query_as::<_, GroupRow>(
            "SELECT id, name, description, created_at, updated_at, created_by
             FROM groups WHERE created_by = ? ORDER BY name"
        )
        .bind(creator_id.to_string())
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        let mut groups = Vec::new();
        for row in rows {
            let contact_ids = sqlx::query_scalar::<_, String>(
                "SELECT contact_id FROM group_members WHERE group_id = ?"
            )
            .bind(row.id.clone())
            .fetch_all(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?
            .into_iter()
            .filter_map(|s| Uuid::parse_str(&s).ok())
            .collect();

            groups.push(row.into_group(contact_ids));
        }

        Ok(groups)
    }
}

#[derive(sqlx::FromRow)]
struct GroupRow {
    id: String,
    name: String,
    description: Option<String>,
    created_at: String,
    updated_at: String,
    created_by: String,
}

impl GroupRow {
    fn into_group(self, contact_ids: Vec<Uuid>) -> Group {
        Group {
            id: Uuid::parse_str(&self.id).unwrap(),
            name: self.name,
            description: self.description,
            contact_ids,
            created_at: chrono::DateTime::parse_from_rfc3339(&self.created_at).unwrap().with_timezone(&chrono::Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(&self.updated_at).unwrap().with_timezone(&chrono::Utc),
            created_by: Uuid::parse_str(&self.created_by).unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup_test_db() -> Pool<Sqlite> {
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
    async fn test_create_and_get_group() {
        let pool = setup_test_db().await;
        let repo = GroupRepository::new(&pool);

        let user_id = Uuid::new_v4();
        sqlx::query("INSERT INTO users (id, email, name, password_hash, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(user_id.to_string())
            .bind("test@example.com")
            .bind("Test")
            .bind("hash")
            .bind(Utc::now().to_rfc3339())
            .bind(Utc::now().to_rfc3339())
            .execute(&pool)
            .await
            .unwrap();

        let group = Group {
            id: Uuid::new_v4(),
            name: "Test Group".to_string(),
            description: Some("A test group".to_string()),
            contact_ids: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: user_id,
        };

        repo.create(&group).await.unwrap();

        let retrieved = repo.get_by_id(group.id).await.unwrap();
        assert_eq!(retrieved.name, group.name);
    }

    #[tokio::test]
    async fn test_add_and_remove_member() {
        let pool = setup_test_db().await;
        let repo = GroupRepository::new(&pool);

        let user_id = Uuid::new_v4();
        sqlx::query("INSERT INTO users (id, email, name, password_hash, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(user_id.to_string())
            .bind("test@example.com")
            .bind("Test")
            .bind("hash")
            .bind(Utc::now().to_rfc3339())
            .bind(Utc::now().to_rfc3339())
            .execute(&pool)
            .await
            .unwrap();

        let contact_id = Uuid::new_v4();
        sqlx::query("INSERT INTO contacts (id, first_name, created_at, updated_at, created_by, version) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(contact_id.to_string())
            .bind("John")
            .bind(Utc::now().to_rfc3339())
            .bind(Utc::now().to_rfc3339())
            .bind(user_id.to_string())
            .bind(1)
            .execute(&pool)
            .await
            .unwrap();

        let group = Group {
            id: Uuid::new_v4(),
            name: "Test Group".to_string(),
            description: None,
            contact_ids: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: user_id,
        };

        repo.create(&group).await.unwrap();
        repo.add_member(group.id, contact_id).await.unwrap();

        let retrieved = repo.get_by_id(group.id).await.unwrap();
        assert_eq!(retrieved.contact_ids.len(), 1);

        repo.remove_member(group.id, contact_id).await.unwrap();

        let retrieved = repo.get_by_id(group.id).await.unwrap();
        assert_eq!(retrieved.contact_ids.len(), 0);
    }
}