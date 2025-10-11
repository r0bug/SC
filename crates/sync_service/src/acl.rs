use core_domain::{AclGrant, Permission, ResourceAcl, ShareEntityType};
use local_store::repositories::ResourceAclRepository;
use sqlx::{Pool, Sqlite};
use std::sync::Arc;
use uuid::Uuid;

pub struct AclService {
    pool: Arc<Pool<Sqlite>>,
}

impl AclService {
    pub fn new(pool: Arc<Pool<Sqlite>>) -> Self {
        Self { pool }
    }

    /// Check if a user has a specific permission on an entity
    pub async fn check_permission(
        &self,
        user_id: &Uuid,
        entity_type: ShareEntityType,
        entity_id: &Uuid,
        permission: Permission,
    ) -> Result<bool, AclError> {
        let repo = ResourceAclRepository::new(&self.pool);

        // Try to get ACL for the entity
        let acl = match repo.get_by_entity(entity_type, *entity_id).await {
            Ok(acl) => acl,
            Err(_) => {
                // No ACL means no permissions
                return Ok(false);
            }
        };

        // Owner has all permissions
        if acl.owner_id == *user_id {
            return Ok(true);
        }

        // Check grants
        for grant in &acl.grants {
            if grant.user_id == *user_id && grant.permissions.contains(&permission) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Check if a user can read an entity
    pub async fn can_read(
        &self,
        user_id: &Uuid,
        entity_type: ShareEntityType,
        entity_id: &Uuid,
    ) -> Result<bool, AclError> {
        self.check_permission(user_id, entity_type, entity_id, Permission::Read)
            .await
    }

    /// Check if a user can write to an entity
    pub async fn can_write(
        &self,
        user_id: &Uuid,
        entity_type: ShareEntityType,
        entity_id: &Uuid,
    ) -> Result<bool, AclError> {
        self.check_permission(user_id, entity_type, entity_id, Permission::Write)
            .await
    }

    /// Check if a user can delete an entity
    pub async fn can_delete(
        &self,
        user_id: &Uuid,
        entity_type: ShareEntityType,
        entity_id: &Uuid,
    ) -> Result<bool, AclError> {
        self.check_permission(user_id, entity_type, entity_id, Permission::Delete)
            .await
    }

    /// Check if a user can share an entity
    pub async fn can_share(
        &self,
        user_id: &Uuid,
        entity_type: ShareEntityType,
        entity_id: &Uuid,
    ) -> Result<bool, AclError> {
        self.check_permission(user_id, entity_type, entity_id, Permission::Share)
            .await
    }

    /// Check if a user is the owner of an entity
    pub async fn is_owner(
        &self,
        user_id: &Uuid,
        entity_type: ShareEntityType,
        entity_id: &Uuid,
    ) -> Result<bool, AclError> {
        let repo = ResourceAclRepository::new(&self.pool);

        let acl = match repo.get_by_entity(entity_type, *entity_id).await {
            Ok(acl) => acl,
            Err(_) => {
                return Ok(false);
            }
        };

        Ok(acl.owner_id == *user_id)
    }

    /// Create a new ACL for an entity (makes the user the owner)
    pub async fn create_acl(
        &self,
        user_id: &Uuid,
        entity_type: ShareEntityType,
        entity_id: &Uuid,
    ) -> Result<(), AclError> {
        let repo = ResourceAclRepository::new(&self.pool);

        let acl = ResourceAcl {
            id: Uuid::new_v4(),
            entity_type,
            entity_id: *entity_id,
            owner_id: *user_id,
            grants: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        repo.create(&acl)
            .await
            .map_err(|_| AclError::InternalError)?;

        Ok(())
    }

    /// Grant permissions to a user
    pub async fn grant_permission(
        &self,
        granter_id: &Uuid,
        grantee_id: &Uuid,
        entity_type: ShareEntityType,
        entity_id: &Uuid,
        permissions: Vec<Permission>,
    ) -> Result<(), AclError> {
        // Check if granter can share
        if !self.can_share(granter_id, entity_type, entity_id).await? {
            return Err(AclError::InsufficientPermission);
        }

        let repo = ResourceAclRepository::new(&self.pool);

        // Get or create ACL
        let acl_id = match repo.get_by_entity(entity_type, *entity_id).await {
            Ok(acl) => acl.id,
            Err(_) => {
                // Create ACL with granter as owner
                self.create_acl(granter_id, entity_type, entity_id).await?;
                repo.get_by_entity(entity_type, *entity_id)
                    .await
                    .map_err(|_| AclError::InternalError)?
                    .id
            }
        };

        let grant = AclGrant {
            user_id: *grantee_id,
            permissions,
            granted_at: chrono::Utc::now(),
            granted_by: *granter_id,
        };

        repo.add_grant(acl_id, &grant)
            .await
            .map_err(|_| AclError::InternalError)?;

        Ok(())
    }

    /// Revoke permissions from a user
    pub async fn revoke_permission(
        &self,
        revoker_id: &Uuid,
        revokee_id: &Uuid,
        entity_type: ShareEntityType,
        entity_id: &Uuid,
    ) -> Result<(), AclError> {
        // Check if revoker is owner or can share
        if !self.is_owner(revoker_id, entity_type, entity_id).await?
            && !self.can_share(revoker_id, entity_type, entity_id).await?
        {
            return Err(AclError::InsufficientPermission);
        }

        let repo = ResourceAclRepository::new(&self.pool);

        let acl = repo
            .get_by_entity(entity_type, *entity_id)
            .await
            .map_err(|_| AclError::NotFound)?;

        repo.remove_grant(acl.id, *revokee_id)
            .await
            .map_err(|_| AclError::InternalError)?;

        Ok(())
    }
}

#[derive(Debug)]
pub enum AclError {
    InsufficientPermission,
    NotFound,
    InternalError,
}

impl From<AclError> for axum::http::StatusCode {
    fn from(err: AclError) -> Self {
        match err {
            AclError::InsufficientPermission => axum::http::StatusCode::FORBIDDEN,
            AclError::NotFound => axum::http::StatusCode::NOT_FOUND,
            AclError::InternalError => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
