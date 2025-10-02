use crate::{auth::AuthUser, state::AppState, validation};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use core_domain::Concept;
use local_store::repositories::ConceptRepository;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateConceptRequest {
    pub name: String,
    pub description: Option<String>,
    pub related_contacts: Vec<Uuid>,
    pub related_projects: Vec<Uuid>,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ConceptResponse {
    pub concept: Concept,
}

/// POST /api/concepts
pub async fn create_concept(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(req): Json<CreateConceptRequest>,
) -> Result<Json<ConceptResponse>, (StatusCode, String)> {
    // Validate inputs
    validation::validate_name(&req.name)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.0))?;
    validation::validate_optional(&req.description, |d: &String| validation::validate_description(d))
        .map_err(|e| (StatusCode::BAD_REQUEST, e.0))?;
    validation::validate_uuid_list(&req.related_contacts, validation::MAX_CONTACTS_COUNT, "related contacts")
        .map_err(|e| (StatusCode::BAD_REQUEST, e.0))?;
    validation::validate_uuid_list(&req.related_projects, validation::MAX_CONTACTS_COUNT, "related projects")
        .map_err(|e| (StatusCode::BAD_REQUEST, e.0))?;
    validation::validate_tags(&req.tags)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.0))?;

    let concept = Concept {
        id: Uuid::new_v4(),
        name: req.name,
        description: req.description,
        related_contacts: req.related_contacts,
        related_projects: req.related_projects,
        tags: req.tags,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        created_by: user.id,
    };

    let repo = ConceptRepository::new(&app_state.pool);
    repo.create(&concept)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create concept".to_string()))?;

    Ok(Json(ConceptResponse { concept }))
}

/// GET /api/concepts/:id
pub async fn get_concept(
    State(app_state): State<AppState>,
    AuthUser(_user): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<ConceptResponse>, (StatusCode, &'static str)> {
    let repo = ConceptRepository::new(&app_state.pool);
    let concept = repo
        .get_by_id(id)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Concept not found"))?;

    Ok(Json(ConceptResponse { concept }))
}

/// PUT /api/concepts/:id
pub async fn update_concept(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateConceptRequest>,
) -> Result<Json<ConceptResponse>, (StatusCode, &'static str)> {
    let repo = ConceptRepository::new(&app_state.pool);
    let mut concept = repo
        .get_by_id(id)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Concept not found"))?;

    // Check if user owns the concept
    if concept.created_by != user.id {
        return Err((StatusCode::FORBIDDEN, "Access denied"));
    }

    concept.name = req.name;
    concept.description = req.description;
    concept.related_contacts = req.related_contacts;
    concept.related_projects = req.related_projects;
    concept.tags = req.tags;
    concept.updated_at = chrono::Utc::now();

    repo.update(&concept)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update concept"))?;

    Ok(Json(ConceptResponse { concept }))
}

/// DELETE /api/concepts/:id
pub async fn delete_concept(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, &'static str)> {
    let repo = ConceptRepository::new(&app_state.pool);
    let concept = repo
        .get_by_id(id)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Concept not found"))?;

    // Check if user owns the concept
    if concept.created_by != user.id {
        return Err((StatusCode::FORBIDDEN, "Access denied"));
    }

    repo.delete(id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete concept"))?;

    Ok(Json(serde_json::json!({ "message": "Concept deleted" })))
}

/// GET /api/concepts
pub async fn list_concepts(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<Vec<Concept>>, (StatusCode, &'static str)> {
    let repo = ConceptRepository::new(&app_state.pool);

    // Return concepts created by the user
    let concepts = repo
        .list_by_creator(user.id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to list concepts"))?;

    Ok(Json(concepts))
}

/// GET /api/concepts/search
pub async fn search_concepts(
    State(app_state): State<AppState>,
    AuthUser(_user): AuthUser,
    Json(query): Json<serde_json::Value>,
) -> Result<Json<Vec<Concept>>, (StatusCode, String)> {
    let repo = ConceptRepository::new(&app_state.pool);

    let search_query = query
        .get("query")
        .and_then(|q| q.as_str())
        .unwrap_or("");

    // Validate query
    validation::validate_query(search_query)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.0))?;

    let concepts = repo
        .search(search_query)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to search concepts".to_string()))?;

    Ok(Json(concepts))
}