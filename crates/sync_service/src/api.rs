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
use serde::Deserialize;
use uuid::Uuid;

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
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Json(contact): Json<Contact>,
) -> Result<(StatusCode, Json<Contact>), StatusCode> {
    let repo = ContactRepository::new(state.store.pool());
    repo.create(&contact)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Broadcast WebSocket event
    state
        .ws_broadcaster
        .broadcast(crate::websocket::BroadcastEvent::ContactCreated {
            id: contact.id,
            user_id: contact.created_by,
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
    Json(tag): Json<Tag>,
) -> Result<(StatusCode, Json<Tag>), StatusCode> {
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
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Json(project): Json<Project>,
) -> Result<(StatusCode, Json<Project>), StatusCode> {
    let repo = ProjectRepository::new(state.store.pool());
    repo.create(&project)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(project)))
}

pub async fn create_note(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Json(note): Json<Note>,
) -> Result<(StatusCode, Json<Note>), StatusCode> {
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
