//! End-to-End Encryption Routes
//!
//! This module provides zero-knowledge encrypted entity storage.
//! The server never has access to decryption keys or plaintext data.
//!
//! Security Model:
//! - Client encrypts data with password-derived key (PBKDF2/Argon2)
//! - Server stores encrypted blobs without ability to decrypt
//! - Integrity verified via SHA-256 checksums
//! - Soft delete support for data recovery

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::Arc;
use tracing::{error, info, warn};

/// Application state for encryption routes
#[derive(Clone)]
pub struct AppState {
    pub db_pool: SqlitePool,
}

/// Request to store an encrypted entity
#[derive(Debug, Deserialize)]
pub struct StoreEncryptedEntityRequest {
    /// Optional entity ID (server generates UUID if not provided)
    pub id: Option<String>,
    /// Entity type: 'contact', 'note', 'project', 'calendar_event'
    pub entity_type: String,
    /// Base64-encoded encrypted blob (nonce.ciphertext)
    pub encrypted_data: String,
    /// SHA-256 checksum for integrity verification
    pub checksum: String,
}

/// Request to update an encrypted entity
#[derive(Debug, Deserialize)]
pub struct UpdateEncryptedEntityRequest {
    /// Updated encrypted data
    pub encrypted_data: String,
    /// Updated checksum
    pub checksum: String,
}

/// Response after storing encrypted entity
#[derive(Debug, Serialize)]
pub struct StoreEncryptedEntityResponse {
    pub id: String,
    pub user_id: String,
    pub entity_type: String,
    pub checksum: String,
    pub created_at: String,
}

/// Encrypted entity retrieved from database
#[derive(Debug, Serialize)]
pub struct EncryptedEntity {
    pub id: String,
    pub user_id: String,
    pub entity_type: String,
    pub encrypted_data: String,
    pub checksum: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<String>,
}

/// Query parameters for listing entities
#[derive(Debug, Deserialize)]
pub struct ListQuery {
    /// Optional filter by entity type
    pub entity_type: Option<String>,
}

/// Store encrypted entity
///
/// POST /api/encrypted/:user_id
pub async fn store_encrypted_entity(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
    Json(payload): Json<StoreEncryptedEntityRequest>,
) -> Result<Json<StoreEncryptedEntityResponse>, AppError> {
    info!(
        "Storing encrypted entity for user: {}, type: {}",
        user_id, payload.entity_type
    );

    // Generate ID if not provided
    let id = payload.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    // Validate entity type
    let valid_types = ["contact", "note", "project", "calendar_event"];
    if !valid_types.contains(&payload.entity_type.as_str()) {
        return Err(AppError::BadRequest(format!(
            "Invalid entity_type. Must be one of: {}",
            valid_types.join(", ")
        )));
    }

    // Insert into database
    let result = sqlx::query(
        r#"
        INSERT INTO encrypted_entities (id, user_id, entity_type, encrypted_data, checksum)
        VALUES (?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(&user_id)
    .bind(&payload.entity_type)
    .bind(&payload.encrypted_data)
    .bind(&payload.checksum)
    .execute(&state.db_pool)
    .await;

    match result {
        Ok(_) => {
            info!("Successfully stored encrypted entity: {}", id);

            // Fetch created entity to get timestamp
            let entity: (String,) = sqlx::query_as(
                "SELECT created_at FROM encrypted_entities WHERE id = ?",
            )
            .bind(&id)
            .fetch_one(&state.db_pool)
            .await
            .map_err(|e| {
                error!("Failed to fetch created entity: {}", e);
                AppError::InternalServerError
            })?;

            Ok(Json(StoreEncryptedEntityResponse {
                id,
                user_id,
                entity_type: payload.entity_type,
                checksum: payload.checksum,
                created_at: entity.0,
            }))
        }
        Err(e) => {
            error!("Failed to store encrypted entity: {}", e);
            Err(AppError::InternalServerError)
        }
    }
}

/// Get encrypted entity by ID
///
/// GET /api/encrypted/:user_id/:entity_id
pub async fn get_encrypted_entity(
    State(state): State<Arc<AppState>>,
    Path((user_id, entity_id)): Path<(String, String)>,
) -> Result<Json<EncryptedEntity>, AppError> {
    info!(
        "Fetching encrypted entity: {} for user: {}",
        entity_id, user_id
    );

    let entity = sqlx::query_as::<_, (String, String, String, String, String, String, String, Option<String>)>(
        r#"
        SELECT id, user_id, entity_type, encrypted_data, checksum, created_at, updated_at, deleted_at
        FROM encrypted_entities
        WHERE id = ? AND user_id = ? AND deleted_at IS NULL
        "#,
    )
    .bind(&entity_id)
    .bind(&user_id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Database error fetching encrypted entity: {}", e);
        AppError::InternalServerError
    })?;

    match entity {
        Some((id, user_id, entity_type, encrypted_data, checksum, created_at, updated_at, deleted_at)) => {
            Ok(Json(EncryptedEntity {
                id,
                user_id,
                entity_type,
                encrypted_data,
                checksum,
                created_at,
                updated_at,
                deleted_at,
            }))
        }
        None => {
            warn!("Encrypted entity not found: {}", entity_id);
            Err(AppError::NotFound)
        }
    }
}

/// List encrypted entities for a user
///
/// GET /api/encrypted/:user_id?entity_type=contact
pub async fn list_encrypted_entities(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
    Query(params): Query<ListQuery>,
) -> Result<Json<Vec<EncryptedEntity>>, AppError> {
    info!("Listing encrypted entities for user: {}", user_id);

    let entities = if let Some(entity_type) = params.entity_type {
        sqlx::query_as::<_, (String, String, String, String, String, String, String, Option<String>)>(
            r#"
            SELECT id, user_id, entity_type, encrypted_data, checksum, created_at, updated_at, deleted_at
            FROM encrypted_entities
            WHERE user_id = ? AND entity_type = ? AND deleted_at IS NULL
            ORDER BY created_at DESC
            "#,
        )
        .bind(&user_id)
        .bind(&entity_type)
        .fetch_all(&state.db_pool)
        .await
    } else {
        sqlx::query_as::<_, (String, String, String, String, String, String, String, Option<String>)>(
            r#"
            SELECT id, user_id, entity_type, encrypted_data, checksum, created_at, updated_at, deleted_at
            FROM encrypted_entities
            WHERE user_id = ? AND deleted_at IS NULL
            ORDER BY created_at DESC
            "#,
        )
        .bind(&user_id)
        .fetch_all(&state.db_pool)
        .await
    }
    .map_err(|e| {
        error!("Database error listing encrypted entities: {}", e);
        AppError::InternalServerError
    })?;

    let result: Vec<EncryptedEntity> = entities
        .into_iter()
        .map(|(id, user_id, entity_type, encrypted_data, checksum, created_at, updated_at, deleted_at)| {
            EncryptedEntity {
                id,
                user_id,
                entity_type,
                encrypted_data,
                checksum,
                created_at,
                updated_at,
                deleted_at,
            }
        })
        .collect();

    info!("Found {} encrypted entities", result.len());
    Ok(Json(result))
}

/// Update encrypted entity
///
/// PUT /api/encrypted/:user_id/:entity_id
pub async fn update_encrypted_entity(
    State(state): State<Arc<AppState>>,
    Path((user_id, entity_id)): Path<(String, String)>,
    Json(payload): Json<UpdateEncryptedEntityRequest>,
) -> Result<Json<StoreEncryptedEntityResponse>, AppError> {
    info!(
        "Updating encrypted entity: {} for user: {}",
        entity_id, user_id
    );

    // Update in database
    let result = sqlx::query(
        r#"
        UPDATE encrypted_entities
        SET encrypted_data = ?, checksum = ?, updated_at = datetime('now')
        WHERE id = ? AND user_id = ? AND deleted_at IS NULL
        "#,
    )
    .bind(&payload.encrypted_data)
    .bind(&payload.checksum)
    .bind(&entity_id)
    .bind(&user_id)
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to update encrypted entity: {}", e);
        AppError::InternalServerError
    })?;

    if result.rows_affected() == 0 {
        warn!("Encrypted entity not found for update: {}", entity_id);
        return Err(AppError::NotFound);
    }

    // Fetch updated entity
    let entity = sqlx::query_as::<_, (String, String, String)>(
        "SELECT entity_type, checksum, created_at FROM encrypted_entities WHERE id = ?",
    )
    .bind(&entity_id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to fetch updated entity: {}", e);
        AppError::InternalServerError
    })?;

    info!("Successfully updated encrypted entity: {}", entity_id);
    Ok(Json(StoreEncryptedEntityResponse {
        id: entity_id,
        user_id,
        entity_type: entity.0,
        checksum: entity.1,
        created_at: entity.2,
    }))
}

/// Delete encrypted entity (soft delete)
///
/// DELETE /api/encrypted/:user_id/:entity_id
pub async fn delete_encrypted_entity(
    State(state): State<Arc<AppState>>,
    Path((user_id, entity_id)): Path<(String, String)>,
) -> Result<StatusCode, AppError> {
    info!(
        "Soft deleting encrypted entity: {} for user: {}",
        entity_id, user_id
    );

    let result = sqlx::query(
        r#"
        UPDATE encrypted_entities
        SET deleted_at = datetime('now')
        WHERE id = ? AND user_id = ? AND deleted_at IS NULL
        "#,
    )
    .bind(&entity_id)
    .bind(&user_id)
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to delete encrypted entity: {}", e);
        AppError::InternalServerError
    })?;

    if result.rows_affected() == 0 {
        warn!("Encrypted entity not found for deletion: {}", entity_id);
        return Err(AppError::NotFound);
    }

    info!("Successfully deleted encrypted entity: {}", entity_id);
    Ok(StatusCode::NO_CONTENT)
}

/// Create router with all encryption routes
pub fn encryption_routes(db_pool: SqlitePool) -> Router {
    let state = Arc::new(AppState { db_pool });

    Router::new()
        .route("/api/encrypted/:user_id", post(store_encrypted_entity))
        .route("/api/encrypted/:user_id", get(list_encrypted_entities))
        .route(
            "/api/encrypted/:user_id/:entity_id",
            get(get_encrypted_entity),
        )
        .route(
            "/api/encrypted/:user_id/:entity_id",
            put(update_encrypted_entity),
        )
        .route(
            "/api/encrypted/:user_id/:entity_id",
            delete(delete_encrypted_entity),
        )
        .with_state(state)
}

/// Application error types
#[derive(Debug)]
pub enum AppError {
    NotFound,
    BadRequest(String),
    InternalServerError,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "Entity not found"),
            AppError::BadRequest(msg) => {
                return (StatusCode::BAD_REQUEST, msg).into_response();
            }
            AppError::InternalServerError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
        };

        (status, message).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_entity_types() {
        let valid_types = ["contact", "note", "project", "calendar_event"];
        assert!(valid_types.contains(&"contact"));
        assert!(valid_types.contains(&"note"));
        assert!(valid_types.contains(&"project"));
        assert!(valid_types.contains(&"calendar_event"));
        assert!(!valid_types.contains(&"invalid"));
    }
}
