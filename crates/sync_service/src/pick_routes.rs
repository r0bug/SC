use crate::{audit, auth::AuthUser, state::AppState, validation};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::get,
    Json, Router,
};
use core_domain::{Pick, PickStatus, ShareEntityType};
use local_store::repositories::PickRepository;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreatePickRequest {
    pub name: String,
    pub description: Option<String>,
    pub status: Option<String>,
    pub date_start: Option<String>,
    pub date_end: Option<String>,
    pub recurrence: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
}

/// Build pick routes as a Router
pub fn pick_routes() -> Router<AppState> {
    Router::new()
        .route("/api/picks", get(list_picks).post(create_pick))
        .route(
            "/api/picks/:id",
            get(get_pick).put(update_pick).delete(delete_pick),
        )
}

/// POST /api/picks
pub async fn create_pick(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    headers: HeaderMap,
    Json(req): Json<CreatePickRequest>,
) -> Result<Json<Pick>, (StatusCode, String)> {
    validation::validate_name(&req.name).map_err(|e| (StatusCode::BAD_REQUEST, e.0))?;
    validation::validate_optional(&req.description, |d: &String| {
        validation::validate_description(d)
    })
    .map_err(|e| (StatusCode::BAD_REQUEST, e.0))?;

    let now = chrono::Utc::now();

    let status = match req.status.as_deref() {
        Some("active") => PickStatus::Active,
        Some("past") => PickStatus::Past,
        Some("recurring") => PickStatus::Recurring,
        _ => PickStatus::Upcoming,
    };

    let date_start = req
        .date_start
        .as_ref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&chrono::Utc));

    let date_end = req
        .date_end
        .as_ref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&chrono::Utc));

    let pick = Pick {
        id: Uuid::new_v4(),
        name: req.name,
        description: req.description,
        status,
        date_start,
        date_end,
        recurrence: req.recurrence,
        metadata: req.metadata.unwrap_or_else(|| serde_json::json!({})),
        created_at: now,
        updated_at: now,
        created_by: user.id,
    };

    let repo = PickRepository::new(&app_state.pool);
    repo.create(&pick).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to create pick".to_string(),
        )
    })?;

    app_state
        .acl_service
        .create_acl(&user.id, ShareEntityType::Pick, &pick.id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create ACL".to_string(),
            )
        })?;

    let ip = audit::extract_ip_address(&headers);
    let user_agent = audit::extract_user_agent(&headers);
    let _ = app_state
        .audit_service
        .log_pick_create(pick.id, user.id, ip, user_agent)
        .await;

    Ok(Json(pick))
}

/// GET /api/picks/:id
pub async fn get_pick(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Pick>, (StatusCode, &'static str)> {
    let repo = PickRepository::new(&app_state.pool);
    let pick = repo
        .get_by_id(id)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Pick not found"))?;

    // Owner check
    if pick.created_by != user.id {
        let can_read = app_state
            .acl_service
            .can_read(&user.id, ShareEntityType::Pick, &pick.id)
            .await
            .unwrap_or(false);
        if !can_read {
            return Err((StatusCode::FORBIDDEN, "Access denied"));
        }
    }

    Ok(Json(pick))
}

/// PUT /api/picks/:id
pub async fn update_pick(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(req): Json<CreatePickRequest>,
) -> Result<Json<Pick>, (StatusCode, &'static str)> {
    validation::validate_name(&req.name).map_err(|_| (StatusCode::BAD_REQUEST, "Invalid name"))?;

    let repo = PickRepository::new(&app_state.pool);
    let mut pick = repo
        .get_by_id(id)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Pick not found"))?;

    // Permission check
    if pick.created_by != user.id {
        let can_write = app_state
            .acl_service
            .can_write(&user.id, ShareEntityType::Pick, &pick.id)
            .await
            .unwrap_or(false);
        if !can_write {
            return Err((StatusCode::FORBIDDEN, "Access denied"));
        }
    }

    let changes = serde_json::json!({
        "name": req.name,
        "description": req.description,
        "status": req.status,
    });

    pick.name = req.name;
    pick.description = req.description;
    if let Some(status_str) = &req.status {
        pick.status = match status_str.as_str() {
            "active" => PickStatus::Active,
            "past" => PickStatus::Past,
            "recurring" => PickStatus::Recurring,
            _ => PickStatus::Upcoming,
        };
    }
    pick.date_start = req
        .date_start
        .as_ref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&chrono::Utc));
    pick.date_end = req
        .date_end
        .as_ref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&chrono::Utc));
    if let Some(recurrence) = req.recurrence {
        pick.recurrence = Some(recurrence);
    }
    if let Some(metadata) = req.metadata {
        pick.metadata = metadata;
    }
    pick.updated_at = chrono::Utc::now();

    repo.update(&pick)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update pick"))?;

    let ip = audit::extract_ip_address(&headers);
    let user_agent = audit::extract_user_agent(&headers);
    let _ = app_state
        .audit_service
        .log_pick_update(id, user.id, changes, ip, user_agent)
        .await;

    Ok(Json(pick))
}

/// DELETE /api/picks/:id
pub async fn delete_pick(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, &'static str)> {
    let repo = PickRepository::new(&app_state.pool);
    let pick = repo
        .get_by_id(id)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Pick not found"))?;

    if pick.created_by != user.id {
        let can_delete = app_state
            .acl_service
            .can_delete(&user.id, ShareEntityType::Pick, &pick.id)
            .await
            .unwrap_or(false);
        if !can_delete {
            return Err((StatusCode::FORBIDDEN, "Access denied"));
        }
    }

    repo.delete(id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete pick"))?;

    let ip = audit::extract_ip_address(&headers);
    let user_agent = audit::extract_user_agent(&headers);
    let _ = app_state
        .audit_service
        .log_pick_delete(id, user.id, ip, user_agent)
        .await;

    Ok(Json(serde_json::json!({ "message": "Pick deleted" })))
}

/// GET /api/picks
pub async fn list_picks(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<Vec<Pick>>, (StatusCode, &'static str)> {
    let repo = PickRepository::new(&app_state.pool);
    let picks = repo
        .list_by_creator(user.id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to list picks"))?;

    Ok(Json(picks))
}
