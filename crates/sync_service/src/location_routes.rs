use crate::{audit, auth::AuthUser, state::AppState, validation};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::get,
    Json, Router,
};
use core_domain::{Location, ShareEntityType};
use local_store::repositories::LocationRepository;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateLocationRequest {
    pub name: String,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip: Option<String>,
    pub coordinates_lat: Option<f64>,
    pub coordinates_lng: Option<f64>,
    pub metadata: Option<serde_json::Value>,
}

/// Build location routes as a Router
pub fn location_routes() -> Router<AppState> {
    Router::new()
        .route("/api/locations", get(list_locations).post(create_location))
        .route(
            "/api/locations/:id",
            get(get_location)
                .put(update_location)
                .delete(delete_location),
        )
}

/// POST /api/locations
pub async fn create_location(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    headers: HeaderMap,
    Json(req): Json<CreateLocationRequest>,
) -> Result<Json<Location>, (StatusCode, String)> {
    validation::validate_name(&req.name).map_err(|e| (StatusCode::BAD_REQUEST, e.0))?;

    let now = chrono::Utc::now();

    let location = Location {
        id: Uuid::new_v4(),
        name: req.name,
        address: req.address,
        city: req.city,
        state: req.state,
        zip: req.zip,
        coordinates_lat: req.coordinates_lat,
        coordinates_lng: req.coordinates_lng,
        metadata: req.metadata.unwrap_or_else(|| serde_json::json!({})),
        created_at: now,
        updated_at: now,
        created_by: user.id,
    };

    let repo = LocationRepository::new(&app_state.pool);
    repo.create(&location).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to create location".to_string(),
        )
    })?;

    app_state
        .acl_service
        .create_acl(&user.id, ShareEntityType::Location, &location.id)
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
        .log_location_create(location.id, user.id, ip, user_agent)
        .await;

    Ok(Json(location))
}

/// GET /api/locations/:id
pub async fn get_location(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Location>, (StatusCode, &'static str)> {
    let repo = LocationRepository::new(&app_state.pool);
    let location = repo
        .get_by_id(id)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Location not found"))?;

    if location.created_by != user.id {
        let can_read = app_state
            .acl_service
            .can_read(&user.id, ShareEntityType::Location, &location.id)
            .await
            .unwrap_or(false);
        if !can_read {
            return Err((StatusCode::FORBIDDEN, "Access denied"));
        }
    }

    Ok(Json(location))
}

/// PUT /api/locations/:id
pub async fn update_location(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateLocationRequest>,
) -> Result<Json<Location>, (StatusCode, &'static str)> {
    validation::validate_name(&req.name).map_err(|_| (StatusCode::BAD_REQUEST, "Invalid name"))?;

    let repo = LocationRepository::new(&app_state.pool);
    let mut location = repo
        .get_by_id(id)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Location not found"))?;

    if location.created_by != user.id {
        let can_write = app_state
            .acl_service
            .can_write(&user.id, ShareEntityType::Location, &location.id)
            .await
            .unwrap_or(false);
        if !can_write {
            return Err((StatusCode::FORBIDDEN, "Access denied"));
        }
    }

    let changes = serde_json::json!({
        "name": req.name,
        "address": req.address,
        "city": req.city,
    });

    location.name = req.name;
    location.address = req.address;
    location.city = req.city;
    location.state = req.state;
    location.zip = req.zip;
    location.coordinates_lat = req.coordinates_lat;
    location.coordinates_lng = req.coordinates_lng;
    if let Some(metadata) = req.metadata {
        location.metadata = metadata;
    }
    location.updated_at = chrono::Utc::now();

    repo.update(&location).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to update location",
        )
    })?;

    let ip = audit::extract_ip_address(&headers);
    let user_agent = audit::extract_user_agent(&headers);
    let _ = app_state
        .audit_service
        .log_location_update(id, user.id, changes, ip, user_agent)
        .await;

    Ok(Json(location))
}

/// DELETE /api/locations/:id
pub async fn delete_location(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, &'static str)> {
    let repo = LocationRepository::new(&app_state.pool);
    let location = repo
        .get_by_id(id)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Location not found"))?;

    if location.created_by != user.id {
        let can_delete = app_state
            .acl_service
            .can_delete(&user.id, ShareEntityType::Location, &location.id)
            .await
            .unwrap_or(false);
        if !can_delete {
            return Err((StatusCode::FORBIDDEN, "Access denied"));
        }
    }

    repo.delete(id).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to delete location",
        )
    })?;

    let ip = audit::extract_ip_address(&headers);
    let user_agent = audit::extract_user_agent(&headers);
    let _ = app_state
        .audit_service
        .log_location_delete(id, user.id, ip, user_agent)
        .await;

    Ok(Json(serde_json::json!({ "message": "Location deleted" })))
}

/// GET /api/locations
pub async fn list_locations(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<Vec<Location>>, (StatusCode, &'static str)> {
    let repo = LocationRepository::new(&app_state.pool);
    let locations = repo.list_by_creator(user.id).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to list locations",
        )
    })?;

    Ok(Json(locations))
}
