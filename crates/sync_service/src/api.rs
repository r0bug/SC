use crate::audit;
use crate::auth::AuthUser;
use crate::state::AppState;
use crate::validation::{self};
use ai_middleware::SuggestionEngine;
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use core_domain::*;
use local_store::*;
use serde::Deserialize;
use std::collections::HashMap;
use uuid::Uuid;

// Request DTOs for creating entities
#[derive(Debug, Deserialize)]
pub struct CreateContactRequest {
    pub first_name: String,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub organization: Option<String>,
    pub title: Option<String>,
    pub notes: Option<String>,
    pub social_handles: Option<Vec<SocialHandle>>,
    pub tags: Option<Vec<Uuid>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTagRequest {
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateNoteRequest {
    pub contact_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub title: String,
    pub content: String,
}

// Update DTOs - only fields that can be updated
#[derive(Debug, Deserialize)]
pub struct UpdateContactRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub organization: Option<String>,
    pub title: Option<String>,
    pub notes: Option<String>,
    pub social_handles: Option<Vec<SocialHandle>>,
    pub tags: Option<Vec<Uuid>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTagRequest {
    pub name: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNoteRequest {
    pub title: Option<String>,
    pub content: Option<String>,
}

#[derive(Deserialize)]
pub struct ListQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}

async fn ensure_entity_permission(
    state: &AppState,
    user_id: &Uuid,
    owner_id: Uuid,
    entity_type: ShareEntityType,
    entity_id: &Uuid,
    permission: Permission,
) -> Result<(), StatusCode> {
    let permitted = match permission {
        Permission::Read => {
            state
                .acl_service
                .can_read(user_id, entity_type, entity_id)
                .await
        }
        Permission::Write => {
            state
                .acl_service
                .can_write(user_id, entity_type, entity_id)
                .await
        }
        Permission::Delete => {
            state
                .acl_service
                .can_delete(user_id, entity_type, entity_id)
                .await
        }
        Permission::Share => {
            state
                .acl_service
                .can_share(user_id, entity_type, entity_id)
                .await
        }
    };

    match permitted {
        Ok(true) => Ok(()),
        Ok(false) => {
            if owner_id == *user_id {
                Ok(())
            } else {
                Err(StatusCode::FORBIDDEN)
            }
        }
        Err(err) => Err(StatusCode::from(err)),
    }
}

async fn fetch_owner_id(
    state: &AppState,
    entity_type: ShareEntityType,
    entity_id: Uuid,
) -> Result<Uuid, StatusCode> {
    match entity_type {
        ShareEntityType::Contact => {
            let repo = ContactRepository::new(state.store.pool());
            let entity = repo
                .get_by_id(entity_id)
                .await
                .map_err(|_| StatusCode::NOT_FOUND)?;
            Ok(entity.created_by)
        }
        ShareEntityType::Project => {
            let repo = ProjectRepository::new(state.store.pool());
            let entity = repo
                .get_by_id(entity_id)
                .await
                .map_err(|_| StatusCode::NOT_FOUND)?;
            Ok(entity.created_by)
        }
        ShareEntityType::Note => {
            let repo = NoteRepository::new(state.store.pool());
            let entity = repo
                .get_by_id(entity_id)
                .await
                .map_err(|_| StatusCode::NOT_FOUND)?;
            Ok(entity.created_by)
        }
        ShareEntityType::CalendarEvent => {
            let repo = CalendarEventRepository::new(state.store.pool());
            let entity = repo
                .get_by_id(entity_id)
                .await
                .map_err(|_| StatusCode::NOT_FOUND)?;
            Ok(entity.created_by)
        }
        ShareEntityType::Group => {
            let repo = GroupRepository::new(state.store.pool());
            let entity = repo
                .get_by_id(entity_id)
                .await
                .map_err(|_| StatusCode::NOT_FOUND)?;
            Ok(entity.created_by)
        }
        ShareEntityType::Concept => {
            let repo = ConceptRepository::new(state.store.pool());
            let entity = repo
                .get_by_id(entity_id)
                .await
                .map_err(|_| StatusCode::NOT_FOUND)?;
            Ok(entity.created_by)
        }
        ShareEntityType::Pick => {
            let repo = PickRepository::new(state.store.pool());
            let entity = repo
                .get_by_id(entity_id)
                .await
                .map_err(|_| StatusCode::NOT_FOUND)?;
            Ok(entity.created_by)
        }
        ShareEntityType::Location => {
            let repo = LocationRepository::new(state.store.pool());
            let entity = repo
                .get_by_id(entity_id)
                .await
                .map_err(|_| StatusCode::NOT_FOUND)?;
            Ok(entity.created_by)
        }
    }
}

pub async fn list_contacts(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Query(params): Query<ListQuery>,
) -> Result<Json<Vec<Contact>>, (StatusCode, Json<serde_json::Value>)> {
    // Validate pagination
    validation::validate_pagination(params.limit, params.offset).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.0})),
        )
    })?;

    let repo = ContactRepository::new(state.store.pool());
    // Filter by user ownership
    let contacts = repo
        .list(params.limit, params.offset, user.id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Internal server error"})),
            )
        })?;
    Ok(Json(contacts))
}

pub async fn create_contact(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateContactRequest>,
) -> Result<(StatusCode, Json<Contact>), StatusCode> {
    let contact = Contact {
        id: Uuid::new_v4(),
        first_name: req.first_name,
        last_name: req.last_name,
        email: req.email,
        phone: req.phone,
        organization: req.organization,
        title: req.title,
        notes: req.notes,
        social_handles: req.social_handles.unwrap_or_default(),
        tags: req.tags.unwrap_or_default(),
        projects: Vec::new(),
        groups: Vec::new(),
        metadata: serde_json::json!({}),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        created_by: user.id,
        version: 1,
        last_synced_at: None,
    };

    let repo = ContactRepository::new(state.store.pool());
    repo.create(&contact)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    state
        .acl_service
        .create_acl(&user.id, ShareEntityType::Contact, &contact.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Audit log: contact created
    let ip = audit::extract_ip_address(&headers);
    let user_agent = audit::extract_user_agent(&headers);
    let _ = state
        .audit_service
        .log_contact_create(contact.id, user.id, ip, user_agent)
        .await;

    // Broadcast WebSocket event
    state
        .ws_broadcaster
        .broadcast(crate::websocket::BroadcastEvent::ContactCreated {
            id: contact.id,
            user_id: user.id,
        })
        .await;

    Ok((StatusCode::CREATED, Json(contact)))
}

pub async fn get_contact(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Contact>, StatusCode> {
    let repo = ContactRepository::new(state.store.pool());
    let contact = repo
        .get_by_id(id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    ensure_entity_permission(
        &state,
        &user.id,
        contact.created_by,
        ShareEntityType::Contact,
        &contact.id,
        Permission::Read,
    )
    .await?;

    Ok(Json(contact))
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub query: String,
}

pub async fn search_contacts(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Json(search): Json<SearchQuery>,
) -> Result<Json<Vec<Contact>>, (StatusCode, Json<serde_json::Value>)> {
    // Validate query
    validation::validate_query(&search.query).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.0})),
        )
    })?;

    let repo = ContactRepository::new(state.store.pool());
    // Filter by user ownership
    let contacts = repo.search(&search.query, user.id).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Internal server error"})),
        )
    })?;
    Ok(Json(contacts))
}

pub async fn list_tags(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<Tag>>, StatusCode> {
    let repo = TagRepository::new(state.store.pool());
    let tags = repo
        .list()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(tags))
}

pub async fn create_tag(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Json(req): Json<CreateTagRequest>,
) -> Result<(StatusCode, Json<Tag>), StatusCode> {
    let tag = Tag {
        id: Uuid::new_v4(),
        name: req.name,
        color: req.color,
        created_at: chrono::Utc::now(),
    };

    let repo = TagRepository::new(state.store.pool());
    repo.create(&tag)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(tag)))
}

pub async fn list_projects(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<Project>>, StatusCode> {
    let repo = ProjectRepository::new(state.store.pool());
    // Filter by user ownership
    let projects = repo
        .list(user.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(projects))
}

pub async fn create_project(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateProjectRequest>,
) -> Result<(StatusCode, Json<Project>), StatusCode> {
    // Parse status from string
    let status = match req.status.as_deref() {
        Some("active") | Some("Active") => ProjectStatus::Active,
        Some("completed") | Some("Completed") => ProjectStatus::Completed,
        Some("archived") | Some("Archived") => ProjectStatus::Archived,
        Some("on_hold") | Some("OnHold") | Some("on hold") => ProjectStatus::OnHold,
        _ => ProjectStatus::Active, // Default to Active
    };

    let project = Project {
        id: Uuid::new_v4(),
        name: req.name,
        description: req.description,
        status,
        contacts: Vec::new(),
        tags: Vec::new(),
        attachment_ids: Vec::new(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        created_by: user.id,
        version: 1,
        last_synced_at: None,
    };

    let repo = ProjectRepository::new(state.store.pool());
    repo.create(&project).await.map_err(|e| {
        tracing::error!("Failed to create project: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    state
        .acl_service
        .create_acl(&user.id, ShareEntityType::Project, &project.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Audit log: project created
    let ip = audit::extract_ip_address(&headers);
    let user_agent = audit::extract_user_agent(&headers);
    let _ = state
        .audit_service
        .log_project_create(project.id, user.id, ip, user_agent)
        .await;

    Ok((StatusCode::CREATED, Json(project)))
}

pub async fn get_project(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Project>, StatusCode> {
    let repo = ProjectRepository::new(state.store.pool());
    let project = repo
        .get_by_id(id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    ensure_entity_permission(
        &state,
        &user.id,
        project.created_by,
        ShareEntityType::Project,
        &project.id,
        Permission::Read,
    )
    .await?;

    Ok(Json(project))
}

pub async fn update_project(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(updates): Json<UpdateProjectRequest>,
) -> Result<Json<Project>, StatusCode> {
    let repo = ProjectRepository::new(state.store.pool());

    // Fetch existing project using PATH id (not body id) - security fix
    let mut project = repo
        .get_by_id(id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    ensure_entity_permission(
        &state,
        &user.id,
        project.created_by,
        ShareEntityType::Project,
        &project.id,
        Permission::Write,
    )
    .await?;

    // Track changes for audit log
    let mut changes = serde_json::Map::new();

    // Apply partial updates
    if let Some(name) = updates.name {
        changes.insert("name".to_string(), serde_json::json!(name));
        project.name = name;
    }
    if let Some(description) = updates.description {
        changes.insert("description".to_string(), serde_json::json!(description));
        project.description = Some(description);
    }
    if let Some(status) = updates.status {
        changes.insert("status".to_string(), serde_json::json!(status));
        // Parse status from string
        project.status = match status.as_str() {
            "active" | "Active" => ProjectStatus::Active,
            "completed" | "Completed" => ProjectStatus::Completed,
            "archived" | "Archived" => ProjectStatus::Archived,
            "on_hold" | "OnHold" | "on hold" => ProjectStatus::OnHold,
            _ => project.status, // Keep existing status if invalid
        };
    }

    project.updated_at = chrono::Utc::now();

    repo.update(&project)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Audit log: project updated
    let ip = audit::extract_ip_address(&headers);
    let user_agent = audit::extract_user_agent(&headers);
    let _ = state
        .audit_service
        .log_project_update(
            project.id,
            user.id,
            serde_json::Value::Object(changes),
            ip,
            user_agent,
        )
        .await;

    Ok(Json(project))
}

pub async fn delete_project(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let repo = ProjectRepository::new(state.store.pool());

    // Fetch existing project to verify ownership before deletion
    let project = repo
        .get_by_id(id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    ensure_entity_permission(
        &state,
        &user.id,
        project.created_by,
        ShareEntityType::Project,
        &project.id,
        Permission::Delete,
    )
    .await?;

    repo.delete(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Audit log: project deleted
    let ip = audit::extract_ip_address(&headers);
    let user_agent = audit::extract_user_agent(&headers);
    let _ = state
        .audit_service
        .log_project_delete(id, user.id, ip, user_agent)
        .await;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn create_note(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateNoteRequest>,
) -> Result<(StatusCode, Json<Note>), StatusCode> {
    if let Some(contact_id) = req.contact_id {
        let contact_repo = ContactRepository::new(state.store.pool());
        let contact = contact_repo
            .get_by_id(contact_id)
            .await
            .map_err(|_| StatusCode::NOT_FOUND)?;
        ensure_entity_permission(
            &state,
            &user.id,
            contact.created_by,
            ShareEntityType::Contact,
            &contact.id,
            Permission::Write,
        )
        .await?;
    }

    if let Some(project_id) = req.project_id {
        let project_repo = ProjectRepository::new(state.store.pool());
        let project = project_repo
            .get_by_id(project_id)
            .await
            .map_err(|_| StatusCode::NOT_FOUND)?;
        ensure_entity_permission(
            &state,
            &user.id,
            project.created_by,
            ShareEntityType::Project,
            &project.id,
            Permission::Write,
        )
        .await?;
    }

    let note = Note {
        id: Uuid::new_v4(),
        contact_id: req.contact_id,
        project_id: req.project_id,
        title: req.title,
        content: req.content,
        attachment_ids: Vec::new(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        created_by: user.id,
        version: 1,
        last_synced_at: None,
    };

    let repo = NoteRepository::new(state.store.pool());
    repo.create(&note)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    state
        .acl_service
        .create_acl(&user.id, ShareEntityType::Note, &note.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Audit log: note created
    let ip = audit::extract_ip_address(&headers);
    let user_agent = audit::extract_user_agent(&headers);
    let _ = state
        .audit_service
        .log_note_create(note.id, user.id, ip, user_agent)
        .await;

    Ok((StatusCode::CREATED, Json(note)))
}

pub async fn list_notes_by_contact(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<Note>>, StatusCode> {
    let contact_repo = ContactRepository::new(state.store.pool());
    let contact = contact_repo
        .get_by_id(id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    ensure_entity_permission(
        &state,
        &user.id,
        contact.created_by,
        ShareEntityType::Contact,
        &contact.id,
        Permission::Read,
    )
    .await?;

    let repo = NoteRepository::new(state.store.pool());
    let notes = repo
        .list_by_contact(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(notes))
}

pub async fn queue_communication(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Json(attempt): Json<CommunicationAttempt>,
) -> Result<(StatusCode, Json<CommunicationAttempt>), StatusCode> {
    let contact_repo = ContactRepository::new(state.store.pool());
    let contact = contact_repo
        .get_by_id(attempt.contact_id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    ensure_entity_permission(
        &state,
        &user.id,
        contact.created_by,
        ShareEntityType::Contact,
        &contact.id,
        Permission::Write,
    )
    .await?;

    let repo = CommunicationRepository::new(state.store.pool());
    repo.create(&attempt)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(attempt)))
}

pub async fn create_share(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Json(invite): Json<ShareInvite>,
) -> Result<(StatusCode, Json<ShareInvite>), StatusCode> {
    if invite.shared_by != user.id {
        return Err(StatusCode::FORBIDDEN);
    }

    let owner_id = fetch_owner_id(&state, invite.entity_type, invite.entity_id).await?;

    ensure_entity_permission(
        &state,
        &user.id,
        owner_id,
        invite.entity_type,
        &invite.entity_id,
        Permission::Share,
    )
    .await?;

    let repo = ShareRepository::new(state.store.pool());
    repo.create(&invite)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(invite)))
}

pub async fn get_suggestions(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(contact_id): Path<Uuid>,
) -> Result<Json<Vec<AiSuggestion>>, StatusCode> {
    let contact_repo = ContactRepository::new(state.store.pool());
    let contact = contact_repo
        .get_by_id(contact_id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    ensure_entity_permission(
        &state,
        &user.id,
        contact.created_by,
        ShareEntityType::Contact,
        &contact.id,
        Permission::Read,
    )
    .await?;

    let engine = SuggestionEngine::new(state.ai_client.read().await.clone());
    let suggestions = engine
        .generate_contact_suggestions(&contact)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(suggestions))
}

pub async fn update_contact(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(updates): Json<UpdateContactRequest>,
) -> Result<Json<Contact>, StatusCode> {
    let repo = ContactRepository::new(state.store.pool());

    // Fetch existing contact using PATH id (not body id) - security fix
    let mut contact = repo
        .get_by_id(id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    ensure_entity_permission(
        &state,
        &user.id,
        contact.created_by,
        ShareEntityType::Contact,
        &contact.id,
        Permission::Write,
    )
    .await?;

    // Track changes for audit log
    let mut changes = serde_json::Map::new();

    // Apply partial updates
    if let Some(first_name) = updates.first_name {
        changes.insert("first_name".to_string(), serde_json::json!(first_name));
        contact.first_name = first_name;
    }
    if let Some(last_name) = updates.last_name {
        changes.insert("last_name".to_string(), serde_json::json!(last_name));
        contact.last_name = Some(last_name);
    }
    if let Some(email) = updates.email {
        changes.insert("email".to_string(), serde_json::json!(email));
        contact.email = Some(email);
    }
    if let Some(phone) = updates.phone {
        changes.insert("phone".to_string(), serde_json::json!(phone));
        contact.phone = Some(phone);
    }
    if let Some(organization) = updates.organization {
        changes.insert("organization".to_string(), serde_json::json!(organization));
        contact.organization = Some(organization);
    }
    if let Some(title) = updates.title {
        changes.insert("title".to_string(), serde_json::json!(title));
        contact.title = Some(title);
    }
    if let Some(notes) = updates.notes {
        changes.insert("notes".to_string(), serde_json::json!(notes));
        contact.notes = Some(notes);
    }
    if let Some(social_handles) = updates.social_handles {
        changes.insert(
            "social_handles".to_string(),
            serde_json::json!(social_handles),
        );
        contact.social_handles = social_handles;
    }
    if let Some(tags) = updates.tags {
        changes.insert("tags".to_string(), serde_json::json!(tags));
        contact.tags = tags;
    }

    contact.updated_at = chrono::Utc::now();

    repo.update(&contact)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Audit log: contact updated
    let ip = audit::extract_ip_address(&headers);
    let user_agent = audit::extract_user_agent(&headers);
    let _ = state
        .audit_service
        .log_contact_update(
            contact.id,
            user.id,
            serde_json::Value::Object(changes),
            ip,
            user_agent,
        )
        .await;

    Ok(Json(contact))
}

pub async fn delete_contact(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let repo = ContactRepository::new(state.store.pool());

    // Fetch existing contact to verify ownership before deletion
    let contact = repo
        .get_by_id(id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    ensure_entity_permission(
        &state,
        &user.id,
        contact.created_by,
        ShareEntityType::Contact,
        &contact.id,
        Permission::Delete,
    )
    .await?;

    repo.delete(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Audit log: contact deleted
    let ip = audit::extract_ip_address(&headers);
    let user_agent = audit::extract_user_agent(&headers);
    let _ = state
        .audit_service
        .log_contact_delete(id, user.id, ip, user_agent)
        .await;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn update_tag(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(updates): Json<UpdateTagRequest>,
) -> Result<Json<Tag>, StatusCode> {
    let repo = TagRepository::new(state.store.pool());

    // Fetch existing tag using PATH id (not body id) - security fix
    let mut tag = repo
        .get_by_id(id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    // Apply partial updates
    if let Some(name) = updates.name {
        tag.name = name;
    }
    if let Some(color) = updates.color {
        tag.color = Some(color);
    }

    repo.update(&tag)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(tag))
}

pub async fn delete_tag(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let repo = TagRepository::new(state.store.pool());
    repo.delete(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn update_note(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(updates): Json<UpdateNoteRequest>,
) -> Result<Json<Note>, StatusCode> {
    let repo = NoteRepository::new(state.store.pool());

    // Fetch existing note using PATH id (not body id) - security fix
    let mut note = repo
        .get_by_id(id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    ensure_entity_permission(
        &state,
        &user.id,
        note.created_by,
        ShareEntityType::Note,
        &note.id,
        Permission::Write,
    )
    .await?;

    // Track changes for audit log
    let mut changes = serde_json::Map::new();

    // Apply partial updates
    if let Some(title) = updates.title {
        changes.insert("title".to_string(), serde_json::json!(title));
        note.title = title;
    }
    if let Some(content) = updates.content {
        changes.insert("content".to_string(), serde_json::json!(content));
        note.content = content;
    }

    note.updated_at = chrono::Utc::now();

    repo.update(&note)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Audit log: note updated
    let ip = audit::extract_ip_address(&headers);
    let user_agent = audit::extract_user_agent(&headers);
    let _ = state
        .audit_service
        .log_note_update(
            note.id,
            user.id,
            serde_json::Value::Object(changes),
            ip,
            user_agent,
        )
        .await;

    Ok(Json(note))
}

pub async fn delete_note(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let repo = NoteRepository::new(state.store.pool());
    let note = repo
        .get_by_id(id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    ensure_entity_permission(
        &state,
        &user.id,
        note.created_by,
        ShareEntityType::Note,
        &note.id,
        Permission::Delete,
    )
    .await?;

    repo.delete(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Audit log: note deleted
    let ip = audit::extract_ip_address(&headers);
    let user_agent = audit::extract_user_agent(&headers);
    let _ = state
        .audit_service
        .log_note_delete(id, user.id, ip, user_agent)
        .await;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct CommunicationListQuery {
    pub contact_id: Option<Uuid>,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

pub async fn list_communication_attempts(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Query(params): Query<CommunicationListQuery>,
) -> Result<Json<Vec<CommunicationAttempt>>, (StatusCode, Json<serde_json::Value>)> {
    // Validate pagination
    validation::validate_pagination(params.limit, params.offset).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.0})),
        )
    })?;

    let repo = CommunicationRepository::new(state.store.pool());
    let attempts = if let Some(contact_id) = params.contact_id {
        repo.list_by_contact(contact_id).await.map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Internal server error"})),
            )
        })?
    } else {
        repo.list_all_attempts(params.limit, params.offset)
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "Internal server error"})),
                )
            })?
    };
    let contact_repo = ContactRepository::new(state.store.pool());
    let mut owner_cache: HashMap<Uuid, Uuid> = HashMap::new();
    let mut filtered = Vec::new();

    for attempt in attempts {
        let owner_id = if let Some(owner) = owner_cache.get(&attempt.contact_id) {
            *owner
        } else {
            let contact = match contact_repo.get_by_id(attempt.contact_id).await {
                Ok(contact) => contact,
                Err(_) => continue,
            };
            owner_cache.insert(attempt.contact_id, contact.created_by);
            contact.created_by
        };

        match ensure_entity_permission(
            &state,
            &user.id,
            owner_id,
            ShareEntityType::Contact,
            &attempt.contact_id,
            Permission::Read,
        )
        .await
        {
            Ok(()) => filtered.push(attempt),
            Err(StatusCode::FORBIDDEN) => continue,
            Err(code) => {
                return Err((
                    code,
                    Json(serde_json::json!({"error": "Failed to authorize communication attempt"})),
                ));
            }
        }
    }

    Ok(Json(filtered))
}

pub async fn cancel_communication(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let repo = CommunicationRepository::new(state.store.pool());
    let attempt = repo.get_attempt(id).await.map_err(|_| {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Not found"})),
        )
    })?;

    let contact_repo = ContactRepository::new(state.store.pool());
    let contact = contact_repo
        .get_by_id(attempt.contact_id)
        .await
        .map_err(|_| {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Contact not found"})),
            )
        })?;

    if let Err(code) = ensure_entity_permission(
        &state,
        &user.id,
        contact.created_by,
        ShareEntityType::Contact,
        &contact.id,
        Permission::Write,
    )
    .await
    {
        return Err((
            code,
            Json(serde_json::json!({"error": "Access denied for communication"})),
        ));
    }

    repo.cancel(id).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to cancel communication"})),
        )
    })?;
    Ok(StatusCode::NO_CONTENT)
}
