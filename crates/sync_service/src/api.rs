use crate::auth::AuthUser;
use crate::state::AppState;
use crate::validation::{self};
use ai_middleware::SuggestionEngine;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use core_domain::*;
use local_store::*;
use serde::{Deserialize, Serialize};
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

pub async fn list_contacts(
    AuthUser(_user): AuthUser,
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
    let contacts = repo.list(params.limit, params.offset).await.map_err(|_| {
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
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Contact>, StatusCode> {
    let repo = ContactRepository::new(state.store.pool());
    let contact = repo
        .get_by_id(id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    Ok(Json(contact))
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub query: String,
}

pub async fn search_contacts(
    AuthUser(_user): AuthUser,
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
    let contacts = repo.search(&search.query).await.map_err(|_| {
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
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<Project>>, StatusCode> {
    let repo = ProjectRepository::new(state.store.pool());
    let projects = repo
        .list()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(projects))
}

pub async fn create_project(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
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
    repo.create(&project)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(project)))
}

pub async fn get_project(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Project>, StatusCode> {
    let repo = ProjectRepository::new(state.store.pool());
    let project = repo
        .get_by_id(id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    Ok(Json(project))
}

pub async fn update_project(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(project): Json<Project>,
) -> Result<Json<Project>, StatusCode> {
    let repo = ProjectRepository::new(state.store.pool());
    repo.update(&project)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(project))
}

pub async fn delete_project(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let repo = ProjectRepository::new(state.store.pool());
    repo.delete(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn create_note(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Json(req): Json<CreateNoteRequest>,
) -> Result<(StatusCode, Json<Note>), StatusCode> {
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
    Ok((StatusCode::CREATED, Json(note)))
}

pub async fn list_notes_by_contact(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<Note>>, StatusCode> {
    let repo = NoteRepository::new(state.store.pool());
    let notes = repo
        .list_by_contact(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(notes))
}

pub async fn queue_communication(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Json(attempt): Json<CommunicationAttempt>,
) -> Result<(StatusCode, Json<CommunicationAttempt>), StatusCode> {
    let repo = CommunicationRepository::new(state.store.pool());
    repo.create(&attempt)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(attempt)))
}

pub async fn create_share(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Json(invite): Json<ShareInvite>,
) -> Result<(StatusCode, Json<ShareInvite>), StatusCode> {
    let repo = ShareRepository::new(state.store.pool());
    repo.create(&invite)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(invite)))
}

pub async fn get_suggestions(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Path(contact_id): Path<Uuid>,
) -> Result<Json<Vec<AiSuggestion>>, StatusCode> {
    let contact_repo = ContactRepository::new(state.store.pool());
    let contact = contact_repo
        .get_by_id(contact_id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let engine = SuggestionEngine::new((*state.ai_client).clone());
    let suggestions = engine
        .generate_contact_suggestions(&contact)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(suggestions))
}

pub async fn update_contact(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(contact): Json<Contact>,
) -> Result<Json<Contact>, StatusCode> {
    let repo = ContactRepository::new(state.store.pool());
    repo.update(&contact)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(contact))
}

pub async fn delete_contact(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let repo = ContactRepository::new(state.store.pool());
    repo.delete(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn update_tag(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(tag): Json<Tag>,
) -> Result<Json<Tag>, StatusCode> {
    let repo = TagRepository::new(state.store.pool());
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
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(note): Json<Note>,
) -> Result<Json<Note>, StatusCode> {
    let repo = NoteRepository::new(state.store.pool());
    repo.update(&note)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(note))
}

pub async fn delete_note(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let repo = NoteRepository::new(state.store.pool());
    repo.delete(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
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
    AuthUser(_user): AuthUser,
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
        repo.list_by_contact(contact_id)
            .await
            .map_err(|_| {
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
    Ok(Json(attempts))
}

pub async fn cancel_communication(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let repo = CommunicationRepository::new(state.store.pool());
    repo.cancel(id).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to cancel communication"})),
        )
    })?;
    Ok(StatusCode::NO_CONTENT)
}
