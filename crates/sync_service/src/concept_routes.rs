use crate::{audit, auth::AuthUser, state::AppState, validation};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use core_domain::{Concept, Permission, ShareEntityType};
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

async fn ensure_concept_permission(
    app_state: &AppState,
    user_id: &Uuid,
    concept_id: &Uuid,
    owner_id: Uuid,
    permission: Permission,
) -> Result<(), StatusCode> {
    let permitted = match permission {
        Permission::Read => {
            app_state
                .acl_service
                .can_read(user_id, ShareEntityType::Concept, concept_id)
                .await
        }
        Permission::Write => {
            app_state
                .acl_service
                .can_write(user_id, ShareEntityType::Concept, concept_id)
                .await
        }
        Permission::Delete => {
            app_state
                .acl_service
                .can_delete(user_id, ShareEntityType::Concept, concept_id)
                .await
        }
        Permission::Share => {
            app_state
                .acl_service
                .can_share(user_id, ShareEntityType::Concept, concept_id)
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

/// POST /api/concepts
pub async fn create_concept(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    headers: HeaderMap,
    Json(req): Json<CreateConceptRequest>,
) -> Result<Json<ConceptResponse>, (StatusCode, String)> {
    // Validate inputs
    validation::validate_name(&req.name).map_err(|e| (StatusCode::BAD_REQUEST, e.0))?;
    validation::validate_optional(&req.description, |d: &String| {
        validation::validate_description(d)
    })
    .map_err(|e| (StatusCode::BAD_REQUEST, e.0))?;
    validation::validate_uuid_list(
        &req.related_contacts,
        validation::MAX_CONTACTS_COUNT,
        "related contacts",
    )
    .map_err(|e| (StatusCode::BAD_REQUEST, e.0))?;
    validation::validate_uuid_list(
        &req.related_projects,
        validation::MAX_CONTACTS_COUNT,
        "related projects",
    )
    .map_err(|e| (StatusCode::BAD_REQUEST, e.0))?;
    validation::validate_tags(&req.tags).map_err(|e| (StatusCode::BAD_REQUEST, e.0))?;

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
    repo.create(&concept).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to create concept".to_string(),
        )
    })?;

    app_state
        .acl_service
        .create_acl(&user.id, ShareEntityType::Concept, &concept.id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create ACL".to_string(),
            )
        })?;

    // Audit log: concept created
    let ip = audit::extract_ip_address(&headers);
    let user_agent = audit::extract_user_agent(&headers);
    let _ = app_state
        .audit_service
        .log_concept_create(concept.id, user.id, ip, user_agent)
        .await;

    Ok(Json(ConceptResponse { concept }))
}

/// GET /api/concepts/:id
pub async fn get_concept(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<ConceptResponse>, (StatusCode, &'static str)> {
    let repo = ConceptRepository::new(&app_state.pool);
    let concept = repo
        .get_by_id(id)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Concept not found"))?;

    ensure_concept_permission(
        &app_state,
        &user.id,
        &concept.id,
        concept.created_by,
        Permission::Read,
    )
    .await
    .map_err(|code| (code, "Access denied"))?;

    Ok(Json(ConceptResponse { concept }))
}

/// PUT /api/concepts/:id
pub async fn update_concept(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateConceptRequest>,
) -> Result<Json<ConceptResponse>, (StatusCode, &'static str)> {
    let repo = ConceptRepository::new(&app_state.pool);
    let mut concept = repo
        .get_by_id(id)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Concept not found"))?;

    ensure_concept_permission(
        &app_state,
        &user.id,
        &concept.id,
        concept.created_by,
        Permission::Write,
    )
    .await
    .map_err(|code| (code, "Access denied"))?;

    // Track changes for audit log
    let changes = serde_json::json!({
        "name": req.name,
        "description": req.description,
        "related_contacts": req.related_contacts,
        "related_projects": req.related_projects,
        "tags": req.tags
    });

    concept.name = req.name;
    concept.description = req.description;
    concept.related_contacts = req.related_contacts;
    concept.related_projects = req.related_projects;
    concept.tags = req.tags;
    concept.updated_at = chrono::Utc::now();

    repo.update(&concept).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to update concept",
        )
    })?;

    // Audit log: concept updated
    let ip = audit::extract_ip_address(&headers);
    let user_agent = audit::extract_user_agent(&headers);
    let _ = app_state
        .audit_service
        .log_concept_update(id, user.id, changes, ip, user_agent)
        .await;

    Ok(Json(ConceptResponse { concept }))
}

/// DELETE /api/concepts/:id
pub async fn delete_concept(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, &'static str)> {
    let repo = ConceptRepository::new(&app_state.pool);
    let concept = repo
        .get_by_id(id)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Concept not found"))?;

    ensure_concept_permission(
        &app_state,
        &user.id,
        &concept.id,
        concept.created_by,
        Permission::Delete,
    )
    .await
    .map_err(|code| (code, "Access denied"))?;

    repo.delete(id).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to delete concept",
        )
    })?;

    // Audit log: concept deleted
    let ip = audit::extract_ip_address(&headers);
    let user_agent = audit::extract_user_agent(&headers);
    let _ = app_state
        .audit_service
        .log_concept_delete(id, user.id, ip, user_agent)
        .await;

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
    AuthUser(user): AuthUser,
    Json(query): Json<serde_json::Value>,
) -> Result<Json<Vec<Concept>>, (StatusCode, String)> {
    let repo = ConceptRepository::new(&app_state.pool);

    let search_query = query.get("query").and_then(|q| q.as_str()).unwrap_or("");

    // Validate query
    validation::validate_query(search_query).map_err(|e| (StatusCode::BAD_REQUEST, e.0))?;

    let concepts = repo.search(search_query).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to search concepts".to_string(),
        )
    })?;

    // Filter concepts to only those created by the user or shared with them
    let mut filtered_concepts = Vec::new();
    for concept in concepts {
        // User owns the concept
        if concept.created_by == user.id {
            filtered_concepts.push(concept);
            continue;
        }
        // Check if user has read access via ACL
        if app_state
            .acl_service
            .can_read(&user.id, ShareEntityType::Concept, &concept.id)
            .await
            .unwrap_or(false)
        {
            filtered_concepts.push(concept);
        }
    }

    Ok(Json(filtered_concepts))
}
