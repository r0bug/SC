use crate::db::DbPool;
use core_domain::{AclGrant, DomainError, DomainResult, ResourceAcl, ShareEntityType};
use uuid::Uuid;

pub struct ResourceAclRepository<'a> {
    pool: &'a DbPool,
}

impl<'a> ResourceAclRepository<'a> {
    pub fn new(pool: &'a DbPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, acl: &ResourceAcl) -> DomainResult<()> {
        let entity_type_str = match acl.entity_type {
            ShareEntityType::Contact => "Contact",
            ShareEntityType::Project => "Project",
            ShareEntityType::Note => "Note",
            ShareEntityType::CalendarEvent => "CalendarEvent",
            ShareEntityType::Group => "Group",
            ShareEntityType::Concept => "Concept",
            ShareEntityType::Pick => "Pick",
            ShareEntityType::Location => "Location",
        };

        sqlx::query(
            "INSERT INTO resource_acls (id, entity_type, entity_id, owner_id, grants, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(acl.id.to_string())
        .bind(entity_type_str)
        .bind(acl.entity_id.to_string())
        .bind(acl.owner_id.to_string())
        .bind(serde_json::to_string(&acl.grants).unwrap())
        .bind(acl.created_at.to_rfc3339())
        .bind(acl.updated_at.to_rfc3339())
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn get_by_id(&self, id: Uuid) -> DomainResult<ResourceAcl> {
        let row = sqlx::query_as::<_, ResourceAclRow>(
            "SELECT id, entity_type, entity_id, owner_id, grants, created_at, updated_at
             FROM resource_acls WHERE id = ?",
        )
        .bind(id.to_string())
        .fetch_optional(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?
        .ok_or_else(|| DomainError::NotFound(format!("ResourceAcl {}", id)))?;

        Ok(row.into())
    }

    pub async fn get_by_entity(
        &self,
        entity_type: ShareEntityType,
        entity_id: Uuid,
    ) -> DomainResult<ResourceAcl> {
        let entity_type_str = match entity_type {
            ShareEntityType::Contact => "Contact",
            ShareEntityType::Project => "Project",
            ShareEntityType::Note => "Note",
            ShareEntityType::CalendarEvent => "CalendarEvent",
            ShareEntityType::Group => "Group",
            ShareEntityType::Concept => "Concept",
            ShareEntityType::Pick => "Pick",
            ShareEntityType::Location => "Location",
        };

        let row = sqlx::query_as::<_, ResourceAclRow>(
            "SELECT id, entity_type, entity_id, owner_id, grants, created_at, updated_at
             FROM resource_acls WHERE entity_type = ? AND entity_id = ?",
        )
        .bind(entity_type_str)
        .bind(entity_id.to_string())
        .fetch_optional(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?
        .ok_or_else(|| {
            DomainError::NotFound(format!("ResourceAcl for {:?} {}", entity_type, entity_id))
        })?;

        Ok(row.into())
    }

    pub async fn update(&self, acl: &ResourceAcl) -> DomainResult<()> {
        let entity_type_str = match acl.entity_type {
            ShareEntityType::Contact => "Contact",
            ShareEntityType::Project => "Project",
            ShareEntityType::Note => "Note",
            ShareEntityType::CalendarEvent => "CalendarEvent",
            ShareEntityType::Group => "Group",
            ShareEntityType::Concept => "Concept",
            ShareEntityType::Pick => "Pick",
            ShareEntityType::Location => "Location",
        };

        sqlx::query(
            "UPDATE resource_acls SET entity_type = ?, entity_id = ?, owner_id = ?, grants = ?, updated_at = ?
             WHERE id = ?"
        )
        .bind(entity_type_str)
        .bind(acl.entity_id.to_string())
        .bind(acl.owner_id.to_string())
        .bind(serde_json::to_string(&acl.grants).unwrap())
        .bind(acl.updated_at.to_rfc3339())
        .bind(acl.id.to_string())
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn delete(&self, id: Uuid) -> DomainResult<()> {
        sqlx::query("DELETE FROM resource_acls WHERE id = ?")
            .bind(id.to_string())
            .execute(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn add_grant(&self, acl_id: Uuid, grant: &AclGrant) -> DomainResult<()> {
        let mut acl = self.get_by_id(acl_id).await?;
        acl.grants.push(grant.clone());
        acl.updated_at = chrono::Utc::now();
        self.update(&acl).await
    }

    pub async fn remove_grant(&self, acl_id: Uuid, user_id: Uuid) -> DomainResult<()> {
        let mut acl = self.get_by_id(acl_id).await?;
        acl.grants.retain(|g| g.user_id != user_id);
        acl.updated_at = chrono::Utc::now();
        self.update(&acl).await
    }
}

#[derive(sqlx::FromRow)]
struct ResourceAclRow {
    id: String,
    entity_type: String,
    entity_id: String,
    owner_id: String,
    grants: String,
    created_at: String,
    updated_at: String,
}

impl From<ResourceAclRow> for ResourceAcl {
    fn from(row: ResourceAclRow) -> Self {
        let entity_type = match row.entity_type.as_str() {
            "Contact" => ShareEntityType::Contact,
            "Project" => ShareEntityType::Project,
            "Note" => ShareEntityType::Note,
            "CalendarEvent" => ShareEntityType::CalendarEvent,
            "Group" => ShareEntityType::Group,
            "Concept" => ShareEntityType::Concept,
            "Pick" => ShareEntityType::Pick,
            "Location" => ShareEntityType::Location,
            _ => ShareEntityType::Contact,
        };

        Self {
            id: Uuid::parse_str(&row.id).unwrap(),
            entity_type,
            entity_id: Uuid::parse_str(&row.entity_id).unwrap(),
            owner_id: Uuid::parse_str(&row.owner_id).unwrap(),
            grants: serde_json::from_str(&row.grants).unwrap_or_default(),
            created_at: chrono::DateTime::parse_from_rfc3339(&row.created_at)
                .unwrap()
                .with_timezone(&chrono::Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(&row.updated_at)
                .unwrap()
                .with_timezone(&chrono::Utc),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use core_domain::Permission;
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
    async fn test_create_and_get_acl() {
        let pool = setup_test_db().await;
        let repo = ResourceAclRepository::new(&pool);

        let owner_id = Uuid::new_v4();
        sqlx::query("INSERT INTO users (id, email, name, password_hash, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(owner_id.to_string())
            .bind("owner@example.com")
            .bind("Owner")
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
            .bind(owner_id.to_string())
            .bind(1)
            .execute(&pool)
            .await
            .unwrap();

        let acl = ResourceAcl {
            id: Uuid::new_v4(),
            entity_type: ShareEntityType::Contact,
            entity_id: contact_id,
            owner_id,
            grants: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        repo.create(&acl).await.unwrap();

        let retrieved = repo.get_by_id(acl.id).await.unwrap();
        assert_eq!(retrieved.owner_id, owner_id);
        assert_eq!(retrieved.grants.len(), 0);
    }

    #[tokio::test]
    async fn test_add_and_remove_grant() {
        let pool = setup_test_db().await;
        let repo = ResourceAclRepository::new(&pool);

        let owner_id = Uuid::new_v4();
        sqlx::query("INSERT INTO users (id, email, name, password_hash, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(owner_id.to_string())
            .bind("owner@example.com")
            .bind("Owner")
            .bind("hash")
            .bind(Utc::now().to_rfc3339())
            .bind(Utc::now().to_rfc3339())
            .execute(&pool)
            .await
            .unwrap();

        let user_id = Uuid::new_v4();
        sqlx::query("INSERT INTO users (id, email, name, password_hash, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(user_id.to_string())
            .bind("user@example.com")
            .bind("User")
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
            .bind(owner_id.to_string())
            .bind(1)
            .execute(&pool)
            .await
            .unwrap();

        let acl = ResourceAcl {
            id: Uuid::new_v4(),
            entity_type: ShareEntityType::Contact,
            entity_id: contact_id,
            owner_id,
            grants: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        repo.create(&acl).await.unwrap();

        let grant = AclGrant {
            user_id,
            permissions: vec![Permission::Read, Permission::Write],
            granted_at: Utc::now(),
            granted_by: owner_id,
        };

        repo.add_grant(acl.id, &grant).await.unwrap();

        let retrieved = repo.get_by_id(acl.id).await.unwrap();
        assert_eq!(retrieved.grants.len(), 1);

        repo.remove_grant(acl.id, user_id).await.unwrap();

        let retrieved = repo.get_by_id(acl.id).await.unwrap();
        assert_eq!(retrieved.grants.len(), 0);
    }
}
