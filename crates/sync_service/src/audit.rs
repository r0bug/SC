use core_domain::{AuditAction, AuditLog, DomainResult, ShareEntityType};
use local_store::repositories::AuditLogRepository;
use local_store::DbPool;
use std::sync::Arc;
use uuid::Uuid;

/// Service for audit logging - tracks all CRUD operations and authentication events
pub struct AuditService {
    pool: Arc<DbPool>,
}

impl AuditService {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }

    /// Log a CRUD operation on an entity
    pub async fn log_operation(
        &self,
        entity_type: ShareEntityType,
        entity_id: Uuid,
        action: AuditAction,
        user_id: Uuid,
        changes: serde_json::Value,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> DomainResult<()> {
        let repo = AuditLogRepository::new(&self.pool);

        let log = AuditLog {
            id: Uuid::new_v4(),
            entity_type,
            entity_id,
            action,
            user_id,
            changes,
            ip_address,
            user_agent,
            created_at: chrono::Utc::now(),
        };

        repo.create(&log).await
    }

    /// Log contact operations
    pub async fn log_contact_create(
        &self,
        contact_id: Uuid,
        user_id: Uuid,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> DomainResult<()> {
        self.log_operation(
            ShareEntityType::Contact,
            contact_id,
            AuditAction::Create,
            user_id,
            serde_json::json!({"action": "contact_created"}),
            ip,
            user_agent,
        )
        .await
    }

    pub async fn log_contact_update(
        &self,
        contact_id: Uuid,
        user_id: Uuid,
        changes: serde_json::Value,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> DomainResult<()> {
        self.log_operation(
            ShareEntityType::Contact,
            contact_id,
            AuditAction::Update,
            user_id,
            changes,
            ip,
            user_agent,
        )
        .await
    }

    pub async fn log_contact_delete(
        &self,
        contact_id: Uuid,
        user_id: Uuid,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> DomainResult<()> {
        self.log_operation(
            ShareEntityType::Contact,
            contact_id,
            AuditAction::Delete,
            user_id,
            serde_json::json!({"action": "contact_deleted"}),
            ip,
            user_agent,
        )
        .await
    }

    /// Log project operations
    pub async fn log_project_create(
        &self,
        project_id: Uuid,
        user_id: Uuid,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> DomainResult<()> {
        self.log_operation(
            ShareEntityType::Project,
            project_id,
            AuditAction::Create,
            user_id,
            serde_json::json!({"action": "project_created"}),
            ip,
            user_agent,
        )
        .await
    }

    pub async fn log_project_update(
        &self,
        project_id: Uuid,
        user_id: Uuid,
        changes: serde_json::Value,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> DomainResult<()> {
        self.log_operation(
            ShareEntityType::Project,
            project_id,
            AuditAction::Update,
            user_id,
            changes,
            ip,
            user_agent,
        )
        .await
    }

    pub async fn log_project_delete(
        &self,
        project_id: Uuid,
        user_id: Uuid,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> DomainResult<()> {
        self.log_operation(
            ShareEntityType::Project,
            project_id,
            AuditAction::Delete,
            user_id,
            serde_json::json!({"action": "project_deleted"}),
            ip,
            user_agent,
        )
        .await
    }

    /// Log group operations
    pub async fn log_group_create(
        &self,
        group_id: Uuid,
        user_id: Uuid,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> DomainResult<()> {
        self.log_operation(
            ShareEntityType::Group,
            group_id,
            AuditAction::Create,
            user_id,
            serde_json::json!({"action": "group_created"}),
            ip,
            user_agent,
        )
        .await
    }

    pub async fn log_group_update(
        &self,
        group_id: Uuid,
        user_id: Uuid,
        changes: serde_json::Value,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> DomainResult<()> {
        self.log_operation(
            ShareEntityType::Group,
            group_id,
            AuditAction::Update,
            user_id,
            changes,
            ip,
            user_agent,
        )
        .await
    }

    pub async fn log_group_delete(
        &self,
        group_id: Uuid,
        user_id: Uuid,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> DomainResult<()> {
        self.log_operation(
            ShareEntityType::Group,
            group_id,
            AuditAction::Delete,
            user_id,
            serde_json::json!({"action": "group_deleted"}),
            ip,
            user_agent,
        )
        .await
    }

    /// Log calendar event operations
    pub async fn log_event_create(
        &self,
        event_id: Uuid,
        user_id: Uuid,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> DomainResult<()> {
        self.log_operation(
            ShareEntityType::CalendarEvent,
            event_id,
            AuditAction::Create,
            user_id,
            serde_json::json!({"action": "event_created"}),
            ip,
            user_agent,
        )
        .await
    }

    pub async fn log_event_update(
        &self,
        event_id: Uuid,
        user_id: Uuid,
        changes: serde_json::Value,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> DomainResult<()> {
        self.log_operation(
            ShareEntityType::CalendarEvent,
            event_id,
            AuditAction::Update,
            user_id,
            changes,
            ip,
            user_agent,
        )
        .await
    }

    pub async fn log_event_delete(
        &self,
        event_id: Uuid,
        user_id: Uuid,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> DomainResult<()> {
        self.log_operation(
            ShareEntityType::CalendarEvent,
            event_id,
            AuditAction::Delete,
            user_id,
            serde_json::json!({"action": "event_deleted"}),
            ip,
            user_agent,
        )
        .await
    }

    /// Log note operations
    pub async fn log_note_create(
        &self,
        note_id: Uuid,
        user_id: Uuid,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> DomainResult<()> {
        self.log_operation(
            ShareEntityType::Note,
            note_id,
            AuditAction::Create,
            user_id,
            serde_json::json!({"action": "note_created"}),
            ip,
            user_agent,
        )
        .await
    }

    pub async fn log_note_update(
        &self,
        note_id: Uuid,
        user_id: Uuid,
        changes: serde_json::Value,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> DomainResult<()> {
        self.log_operation(
            ShareEntityType::Note,
            note_id,
            AuditAction::Update,
            user_id,
            changes,
            ip,
            user_agent,
        )
        .await
    }

    pub async fn log_note_delete(
        &self,
        note_id: Uuid,
        user_id: Uuid,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> DomainResult<()> {
        self.log_operation(
            ShareEntityType::Note,
            note_id,
            AuditAction::Delete,
            user_id,
            serde_json::json!({"action": "note_deleted"}),
            ip,
            user_agent,
        )
        .await
    }

    /// Log concept operations
    pub async fn log_concept_create(
        &self,
        concept_id: Uuid,
        user_id: Uuid,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> DomainResult<()> {
        self.log_operation(
            ShareEntityType::Concept,
            concept_id,
            AuditAction::Create,
            user_id,
            serde_json::json!({"action": "concept_created"}),
            ip,
            user_agent,
        )
        .await
    }

    pub async fn log_concept_update(
        &self,
        concept_id: Uuid,
        user_id: Uuid,
        changes: serde_json::Value,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> DomainResult<()> {
        self.log_operation(
            ShareEntityType::Concept,
            concept_id,
            AuditAction::Update,
            user_id,
            changes,
            ip,
            user_agent,
        )
        .await
    }

    pub async fn log_concept_delete(
        &self,
        concept_id: Uuid,
        user_id: Uuid,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> DomainResult<()> {
        self.log_operation(
            ShareEntityType::Concept,
            concept_id,
            AuditAction::Delete,
            user_id,
            serde_json::json!({"action": "concept_deleted"}),
            ip,
            user_agent,
        )
        .await
    }

    /// Log pick operations
    pub async fn log_pick_create(
        &self,
        pick_id: Uuid,
        user_id: Uuid,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> DomainResult<()> {
        self.log_operation(
            ShareEntityType::Pick,
            pick_id,
            AuditAction::Create,
            user_id,
            serde_json::json!({"action": "pick_created"}),
            ip,
            user_agent,
        )
        .await
    }

    pub async fn log_pick_update(
        &self,
        pick_id: Uuid,
        user_id: Uuid,
        changes: serde_json::Value,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> DomainResult<()> {
        self.log_operation(
            ShareEntityType::Pick,
            pick_id,
            AuditAction::Update,
            user_id,
            changes,
            ip,
            user_agent,
        )
        .await
    }

    pub async fn log_pick_delete(
        &self,
        pick_id: Uuid,
        user_id: Uuid,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> DomainResult<()> {
        self.log_operation(
            ShareEntityType::Pick,
            pick_id,
            AuditAction::Delete,
            user_id,
            serde_json::json!({"action": "pick_deleted"}),
            ip,
            user_agent,
        )
        .await
    }

    /// Log location operations
    pub async fn log_location_create(
        &self,
        location_id: Uuid,
        user_id: Uuid,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> DomainResult<()> {
        self.log_operation(
            ShareEntityType::Location,
            location_id,
            AuditAction::Create,
            user_id,
            serde_json::json!({"action": "location_created"}),
            ip,
            user_agent,
        )
        .await
    }

    pub async fn log_location_update(
        &self,
        location_id: Uuid,
        user_id: Uuid,
        changes: serde_json::Value,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> DomainResult<()> {
        self.log_operation(
            ShareEntityType::Location,
            location_id,
            AuditAction::Update,
            user_id,
            changes,
            ip,
            user_agent,
        )
        .await
    }

    pub async fn log_location_delete(
        &self,
        location_id: Uuid,
        user_id: Uuid,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> DomainResult<()> {
        self.log_operation(
            ShareEntityType::Location,
            location_id,
            AuditAction::Delete,
            user_id,
            serde_json::json!({"action": "location_deleted"}),
            ip,
            user_agent,
        )
        .await
    }

    /// Log authentication events (login, logout, failed attempts)
    /// Uses Contact entity type as placeholder since auth events don't fit entity model
    pub async fn log_login(
        &self,
        user_id: Uuid,
        success: bool,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> DomainResult<()> {
        let action_data = if success {
            serde_json::json!({"event": "login_success"})
        } else {
            serde_json::json!({"event": "login_failed"})
        };

        self.log_operation(
            ShareEntityType::Contact, // Using Contact as placeholder for auth events
            user_id,                  // entity_id = user_id for auth events
            AuditAction::Read,        // Using Read as placeholder for authentication
            user_id,
            action_data,
            ip,
            user_agent,
        )
        .await
    }

    pub async fn log_signup(
        &self,
        user_id: Uuid,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> DomainResult<()> {
        self.log_operation(
            ShareEntityType::Contact,
            user_id,
            AuditAction::Create,
            user_id,
            serde_json::json!({"event": "user_signup"}),
            ip,
            user_agent,
        )
        .await
    }

    pub async fn log_logout(
        &self,
        user_id: Uuid,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> DomainResult<()> {
        self.log_operation(
            ShareEntityType::Contact,
            user_id,
            AuditAction::Read,
            user_id,
            serde_json::json!({"event": "user_logout"}),
            ip,
            user_agent,
        )
        .await
    }

    /// Retrieve audit logs for an entity
    #[allow(dead_code)]
    pub async fn get_entity_audit_trail(
        &self,
        entity_type: ShareEntityType,
        entity_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> DomainResult<Vec<AuditLog>> {
        let repo = AuditLogRepository::new(&self.pool);
        repo.list_by_entity(entity_type, entity_id, limit, offset)
            .await
    }

    /// Retrieve audit logs for a user
    #[allow(dead_code)]
    pub async fn get_user_audit_trail(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> DomainResult<Vec<AuditLog>> {
        let repo = AuditLogRepository::new(&self.pool);
        repo.list_by_user(user_id, limit, offset).await
    }
}

/// Helper function to extract IP address from headers
pub fn extract_ip_address(headers: &axum::http::HeaderMap) -> Option<String> {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(String::from)
        })
}

/// Helper function to extract user agent from headers
pub fn extract_user_agent(headers: &axum::http::HeaderMap) -> Option<String> {
    headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(String::from)
}
