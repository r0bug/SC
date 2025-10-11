use crate::{auth::AuthUser, state::AppState, validation};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use core_domain::SearchHistory;
use local_store::repositories::SearchHistoryRepository;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct SearchHistoryResponse {
    pub history: Vec<SearchHistory>,
}

/// GET /api/search-history
pub async fn list_search_history(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<SearchHistoryResponse>, (StatusCode, String)> {
    let repo = SearchHistoryRepository::new(&app_state.pool);

    // Validate pagination (hardcoded limit of 100)
    validation::validate_pagination(100, 0).map_err(|e| (StatusCode::BAD_REQUEST, e.0))?;

    let history = repo
        .list_by_user(user.id, 100, 0) // Default to 100 items, no offset
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to list search history".to_string(),
            )
        })?;

    Ok(Json(SearchHistoryResponse { history }))
}

/// GET /api/search-history/:id
pub async fn get_search_history(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let repo = SearchHistoryRepository::new(&app_state.pool);

    let history = repo
        .get_by_id(id)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Search history not found"))?;

    // Check if the history belongs to the user
    if history.user_id != user.id {
        return Err((StatusCode::FORBIDDEN, "Access denied"));
    }

    Ok(Json(history))
}

/// PUT /api/search-history/:id/clicked
pub async fn update_clicked_result(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let clicked_result_id = body
        .get("clicked_result_id")
        .and_then(|id| id.as_str())
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or((StatusCode::BAD_REQUEST, "Invalid clicked_result_id"))?;

    let repo = SearchHistoryRepository::new(&app_state.pool);

    // Verify the search history belongs to the user
    let history = repo
        .get_by_id(id)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Search history not found"))?;

    if history.user_id != user.id {
        return Err((StatusCode::FORBIDDEN, "Access denied"));
    }

    repo.update_clicked_result(id, clicked_result_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to update clicked result",
            )
        })?;

    Ok(Json(
        serde_json::json!({ "message": "Updated clicked result" }),
    ))
}

/// DELETE /api/search-history/:id
pub async fn delete_search_history(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let repo = SearchHistoryRepository::new(&app_state.pool);

    // Verify the search history belongs to the user
    let history = repo
        .get_by_id(id)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Search history not found"))?;

    if history.user_id != user.id {
        return Err((StatusCode::FORBIDDEN, "Access denied"));
    }

    repo.delete(id).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to delete search history",
        )
    })?;

    Ok(Json(
        serde_json::json!({ "message": "Search history deleted" }),
    ))
}

/// DELETE /api/search-history
pub async fn clear_search_history(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<serde_json::Value>, (StatusCode, &'static str)> {
    let repo = SearchHistoryRepository::new(&app_state.pool);

    repo.clear_user_history(user.id).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to clear search history",
        )
    })?;

    Ok(Json(
        serde_json::json!({ "message": "Search history cleared" }),
    ))
}
