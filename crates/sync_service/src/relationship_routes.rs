use crate::{auth::AuthUser, state::AppState, validation};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use core_domain::{EntityRelationship, EntityType, RelationshipType};
use local_store::repositories::{EntityRelationshipRepository, RelationshipTypeRepository};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateRelationshipTypeRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRelationshipRequest {
    pub source_type: String,
    pub source_id: Uuid,
    pub relationship_type_id: Uuid,
    pub target_type: String,
    pub target_id: Uuid,
    pub metadata: Option<serde_json::Value>,
}

/// Build relationship routes as a Router
pub fn relationship_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/relationships",
            get(list_relationships).post(create_relationship),
        )
        .route(
            "/api/relationships/:id",
            get(get_relationship).delete(delete_relationship),
        )
        .route(
            "/api/relationships/types",
            get(list_relationship_types).post(create_relationship_type),
        )
        .route(
            "/api/relationships/types/:id",
            axum::routing::delete(delete_relationship_type),
        )
        .route(
            "/api/relationships/source/:source_type/:source_id",
            get(list_by_source),
        )
        .route(
            "/api/relationships/target/:target_type/:target_id",
            get(list_by_target),
        )
}

fn parse_entity_type(s: &str) -> Result<EntityType, (StatusCode, &'static str)> {
    s.parse::<EntityType>()
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid entity type"))
}

/// GET /api/relationships/types
pub async fn list_relationship_types(
    State(app_state): State<AppState>,
    AuthUser(_user): AuthUser,
) -> Result<Json<Vec<RelationshipType>>, (StatusCode, &'static str)> {
    let repo = RelationshipTypeRepository::new(&app_state.pool);
    let types = repo.list().await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to list relationship types",
        )
    })?;

    Ok(Json(types))
}

/// POST /api/relationships/types
pub async fn create_relationship_type(
    State(app_state): State<AppState>,
    AuthUser(_user): AuthUser,
    Json(req): Json<CreateRelationshipTypeRequest>,
) -> Result<Json<RelationshipType>, (StatusCode, String)> {
    validation::validate_name(&req.name).map_err(|e| (StatusCode::BAD_REQUEST, e.0))?;

    let rt = RelationshipType {
        id: Uuid::new_v4(),
        name: req.name,
        description: req.description,
        is_core: false,
        created_at: chrono::Utc::now(),
    };

    let repo = RelationshipTypeRepository::new(&app_state.pool);
    repo.create(&rt).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to create relationship type".to_string(),
        )
    })?;

    Ok(Json(rt))
}

/// DELETE /api/relationships/types/:id
pub async fn delete_relationship_type(
    State(app_state): State<AppState>,
    AuthUser(_user): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, &'static str)> {
    let repo = RelationshipTypeRepository::new(&app_state.pool);

    // Verify it exists and is not core
    let rt = repo
        .get_by_id(id)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Relationship type not found"))?;

    if rt.is_core {
        return Err((
            StatusCode::FORBIDDEN,
            "Cannot delete core relationship types",
        ));
    }

    repo.delete(id).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to delete relationship type",
        )
    })?;

    Ok(Json(
        serde_json::json!({ "message": "Relationship type deleted" }),
    ))
}

/// POST /api/relationships
pub async fn create_relationship(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(req): Json<CreateRelationshipRequest>,
) -> Result<Json<EntityRelationship>, (StatusCode, String)> {
    let source_type = parse_entity_type(&req.source_type).map_err(|e| (e.0, e.1.to_string()))?;
    let target_type = parse_entity_type(&req.target_type).map_err(|e| (e.0, e.1.to_string()))?;

    // Verify relationship type exists
    let type_repo = RelationshipTypeRepository::new(&app_state.pool);
    let rel_type = type_repo
        .get_by_id(req.relationship_type_id)
        .await
        .map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                "Relationship type not found".to_string(),
            )
        })?;

    let relationship = EntityRelationship {
        id: Uuid::new_v4(),
        source_type,
        source_id: req.source_id,
        relationship_type_id: req.relationship_type_id,
        relationship_type_name: Some(rel_type.name),
        target_type,
        target_id: req.target_id,
        metadata: req.metadata.unwrap_or_else(|| serde_json::json!({})),
        created_at: chrono::Utc::now(),
        created_by: user.id,
    };

    let repo = EntityRelationshipRepository::new(&app_state.pool);
    repo.create(&relationship).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to create relationship".to_string(),
        )
    })?;

    Ok(Json(relationship))
}

/// GET /api/relationships/:id
pub async fn get_relationship(
    State(app_state): State<AppState>,
    AuthUser(_user): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<EntityRelationship>, (StatusCode, &'static str)> {
    let repo = EntityRelationshipRepository::new(&app_state.pool);
    let rel = repo
        .get_by_id(id)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Relationship not found"))?;

    Ok(Json(rel))
}

/// DELETE /api/relationships/:id
pub async fn delete_relationship(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, &'static str)> {
    let repo = EntityRelationshipRepository::new(&app_state.pool);
    let rel = repo
        .get_by_id(id)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Relationship not found"))?;

    // Only the creator can delete
    if rel.created_by != user.id {
        return Err((StatusCode::FORBIDDEN, "Access denied"));
    }

    repo.delete(id).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to delete relationship",
        )
    })?;

    Ok(Json(
        serde_json::json!({ "message": "Relationship deleted" }),
    ))
}

/// GET /api/relationships
pub async fn list_relationships(
    State(app_state): State<AppState>,
    AuthUser(_user): AuthUser,
    axum::extract::Query(params): axum::extract::Query<ListRelationshipsParams>,
) -> Result<Json<Vec<EntityRelationship>>, (StatusCode, &'static str)> {
    let repo = EntityRelationshipRepository::new(&app_state.pool);

    let relationships = if let Some(type_id) = params.type_id {
        repo.list_by_type(type_id).await.map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to list relationships",
            )
        })?
    } else {
        // Return empty if no filter specified (too broad otherwise)
        Vec::new()
    };

    Ok(Json(relationships))
}

#[derive(Debug, Deserialize)]
pub struct ListRelationshipsParams {
    pub type_id: Option<Uuid>,
}

/// GET /api/relationships/source/:source_type/:source_id
pub async fn list_by_source(
    State(app_state): State<AppState>,
    AuthUser(_user): AuthUser,
    Path((source_type_str, source_id)): Path<(String, Uuid)>,
) -> Result<Json<Vec<EntityRelationship>>, (StatusCode, &'static str)> {
    let source_type = parse_entity_type(&source_type_str)?;

    let repo = EntityRelationshipRepository::new(&app_state.pool);
    let relationships = repo
        .list_by_source(&source_type, source_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to list relationships",
            )
        })?;

    Ok(Json(relationships))
}

/// GET /api/relationships/target/:target_type/:target_id
pub async fn list_by_target(
    State(app_state): State<AppState>,
    AuthUser(_user): AuthUser,
    Path((target_type_str, target_id)): Path<(String, Uuid)>,
) -> Result<Json<Vec<EntityRelationship>>, (StatusCode, &'static str)> {
    let target_type = parse_entity_type(&target_type_str)?;

    let repo = EntityRelationshipRepository::new(&app_state.pool);
    let relationships = repo
        .list_by_target(&target_type, target_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to list relationships",
            )
        })?;

    Ok(Json(relationships))
}
