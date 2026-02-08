use crate::{auth::AuthUser, concept_detection::ConceptDetector, state::AppState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use core_domain::{CommunicationConcept, SuggestionStatus};
use local_store::repositories::CommunicationConceptRepository;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct UpdateStatusRequest {
    pub status: String, // "confirmed" or "denied"
}

#[derive(Debug, Deserialize)]
pub struct DetectRequest {
    pub text: String,
}

/// Build communication concept routes as a Router
pub fn communication_concept_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/communications/:comm_type/:comm_id/concepts",
            get(list_by_communication),
        )
        .route(
            "/api/communications/:comm_type/:comm_id/concepts/detect",
            axum::routing::post(detect_concepts),
        )
        .route("/api/communication-concepts/pending", get(list_pending))
        .route(
            "/api/communication-concepts/:id",
            axum::routing::put(update_status),
        )
}

/// GET /api/communications/:comm_type/:comm_id/concepts
pub async fn list_by_communication(
    State(app_state): State<AppState>,
    AuthUser(_user): AuthUser,
    Path((comm_type, comm_id)): Path<(String, Uuid)>,
) -> Result<Json<Vec<CommunicationConcept>>, (StatusCode, &'static str)> {
    let repo = CommunicationConceptRepository::new(&app_state.pool);
    let concepts = repo
        .list_by_communication(&comm_type, comm_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to list communication concepts",
            )
        })?;

    Ok(Json(concepts))
}

/// POST /api/communications/:comm_type/:comm_id/concepts/detect
///
/// Scan the provided text for concept keywords and create suggestions.
pub async fn detect_concepts(
    State(app_state): State<AppState>,
    AuthUser(_user): AuthUser,
    Path((comm_type, comm_id)): Path<(String, Uuid)>,
    Json(req): Json<DetectRequest>,
) -> Result<Json<Vec<CommunicationConcept>>, (StatusCode, String)> {
    let detector = ConceptDetector::new(&app_state.pool);

    let created = detector
        .scan_and_store(&comm_type, comm_id, &req.text)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(created))
}

/// GET /api/communication-concepts/pending
pub async fn list_pending(
    State(app_state): State<AppState>,
    AuthUser(_user): AuthUser,
) -> Result<Json<Vec<CommunicationConcept>>, (StatusCode, &'static str)> {
    let repo = CommunicationConceptRepository::new(&app_state.pool);
    let pending = repo.list_pending().await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to list pending concepts",
        )
    })?;

    Ok(Json(pending))
}

/// PUT /api/communication-concepts/:id
pub async fn update_status(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateStatusRequest>,
) -> Result<Json<CommunicationConcept>, (StatusCode, String)> {
    let status = match req.status.to_lowercase().as_str() {
        "confirmed" => SuggestionStatus::Confirmed,
        "denied" => SuggestionStatus::Denied,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                "Status must be 'confirmed' or 'denied'".to_string(),
            ))
        }
    };

    let repo = CommunicationConceptRepository::new(&app_state.pool);

    // Verify it exists
    let _existing = repo.get_by_id(id).await.map_err(|_| {
        (
            StatusCode::NOT_FOUND,
            "Communication concept not found".to_string(),
        )
    })?;

    repo.update_status(id, &status, user.id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to update status".to_string(),
            )
        })?;

    // Return updated record
    let updated = repo.get_by_id(id).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to fetch updated record".to_string(),
        )
    })?;

    Ok(Json(updated))
}
