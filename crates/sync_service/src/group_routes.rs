use crate::{audit, auth::AuthUser, state::AppState, validation};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use core_domain::Group;
use local_store::repositories::GroupRepository;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateGroupRequest {
    pub name: String,
    pub description: Option<String>,
    pub contact_ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct AddMemberRequest {
    pub contact_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct GroupResponse {
    pub group: Group,
}

/// POST /api/groups
pub async fn create_group(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    headers: HeaderMap,
    Json(req): Json<CreateGroupRequest>,
) -> Result<Json<GroupResponse>, (StatusCode, String)> {
    // Validate inputs
    validation::validate_name(&req.name).map_err(|e| (StatusCode::BAD_REQUEST, e.0))?;
    validation::validate_optional(&req.description, |d: &String| {
        validation::validate_description(d)
    })
    .map_err(|e| (StatusCode::BAD_REQUEST, e.0))?;
    validation::validate_uuid_list(&req.contact_ids, validation::MAX_CONTACTS_COUNT, "contacts")
        .map_err(|e| (StatusCode::BAD_REQUEST, e.0))?;

    let group = Group {
        id: Uuid::new_v4(),
        name: req.name,
        description: req.description,
        contact_ids: req.contact_ids,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        created_by: user.id,
    };

    let repo = GroupRepository::new(&app_state.pool);
    repo.create(&group).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to create group".to_string(),
        )
    })?;

    // Create ACL for the group
    app_state
        .acl_service
        .create_acl(&user.id, core_domain::ShareEntityType::Group, &group.id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create ACL".to_string(),
            )
        })?;

    // Audit log: group created
    let ip = audit::extract_ip_address(&headers);
    let user_agent = audit::extract_user_agent(&headers);
    let _ = app_state
        .audit_service
        .log_group_create(group.id, user.id, ip, user_agent)
        .await;

    Ok(Json(GroupResponse { group }))
}

/// GET /api/groups/:id
pub async fn get_group(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    // Check permission
    if !app_state
        .acl_service
        .can_read(&user.id, core_domain::ShareEntityType::Group, &id)
        .await
        .unwrap_or(false)
    {
        return Err((StatusCode::FORBIDDEN, "Access denied"));
    }

    let repo = GroupRepository::new(&app_state.pool);
    let group = repo
        .get_by_id(id)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Group not found"))?;

    Ok(Json(GroupResponse { group }))
}

/// PUT /api/groups/:id
pub async fn update_group(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateGroupRequest>,
) -> impl IntoResponse {
    // Check permission
    if !app_state
        .acl_service
        .can_write(&user.id, core_domain::ShareEntityType::Group, &id)
        .await
        .unwrap_or(false)
    {
        return Err((StatusCode::FORBIDDEN, "Access denied"));
    }

    let repo = GroupRepository::new(&app_state.pool);
    let mut group = repo
        .get_by_id(id)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Group not found"))?;

    // Track changes for audit log
    let changes = serde_json::json!({
        "name": req.name,
        "description": req.description,
        "contact_ids": req.contact_ids
    });

    group.name = req.name;
    group.description = req.description;
    group.contact_ids = req.contact_ids;
    group.updated_at = chrono::Utc::now();

    repo.update(&group)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update group"))?;

    // Audit log: group updated
    let ip = audit::extract_ip_address(&headers);
    let user_agent = audit::extract_user_agent(&headers);
    let _ = app_state
        .audit_service
        .log_group_update(id, user.id, changes, ip, user_agent)
        .await;

    Ok(Json(GroupResponse { group }))
}

/// DELETE /api/groups/:id
pub async fn delete_group(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    // Check permission
    if !app_state
        .acl_service
        .can_delete(&user.id, core_domain::ShareEntityType::Group, &id)
        .await
        .unwrap_or(false)
    {
        return Err((StatusCode::FORBIDDEN, "Access denied"));
    }

    let repo = GroupRepository::new(&app_state.pool);
    repo.delete(id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete group"))?;

    // Audit log: group deleted
    let ip = audit::extract_ip_address(&headers);
    let user_agent = audit::extract_user_agent(&headers);
    let _ = app_state
        .audit_service
        .log_group_delete(id, user.id, ip, user_agent)
        .await;

    Ok(Json(serde_json::json!({ "message": "Group deleted" })))
}

/// GET /api/groups
pub async fn list_groups(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<Vec<Group>>, (StatusCode, &'static str)> {
    let repo = GroupRepository::new(&app_state.pool);

    // Get groups created by the user
    let mut groups = repo
        .list_by_creator(user.id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to list groups"))?;

    // Also include groups shared with the user via ACL
    // Get all groups and filter by ACL access (using large limit for simplicity)
    let all_groups = repo
        .list(10000, 0)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to list groups"))?;

    let owned_ids: std::collections::HashSet<_> = groups.iter().map(|g| g.id).collect();

    for group in all_groups {
        // Skip if already in the list (owned by user)
        if owned_ids.contains(&group.id) {
            continue;
        }
        // Check if user has read access via ACL
        if app_state
            .acl_service
            .can_read(&user.id, core_domain::ShareEntityType::Group, &group.id)
            .await
            .unwrap_or(false)
        {
            groups.push(group);
        }
    }

    Ok(Json(groups))
}

/// POST /api/groups/:id/members/:contact_id (path parameter version)
pub async fn add_member(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    Path((group_id, contact_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, &'static str)> {
    // Check permission
    if !app_state
        .acl_service
        .can_write(&user.id, core_domain::ShareEntityType::Group, &group_id)
        .await
        .unwrap_or(false)
    {
        return Err((StatusCode::FORBIDDEN, "Access denied"));
    }

    let repo = GroupRepository::new(&app_state.pool);
    repo.add_member(group_id, contact_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to add member"))?;

    Ok(Json(serde_json::json!({ "message": "Member added" })))
}

/// POST /api/groups/:id/members (JSON body version for web client)
pub async fn add_member_json(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(group_id): Path<Uuid>,
    Json(request): Json<AddMemberRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, &'static str)> {
    // Check permission
    if !app_state
        .acl_service
        .can_write(&user.id, core_domain::ShareEntityType::Group, &group_id)
        .await
        .unwrap_or(false)
    {
        return Err((StatusCode::FORBIDDEN, "Access denied"));
    }

    let repo = GroupRepository::new(&app_state.pool);
    repo.add_member(group_id, request.contact_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to add member"))?;

    Ok(Json(serde_json::json!({ "message": "Member added" })))
}

/// DELETE /api/groups/:id/members/:contact_id
pub async fn remove_member(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    Path((group_id, contact_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, &'static str)> {
    // Check permission
    if !app_state
        .acl_service
        .can_write(&user.id, core_domain::ShareEntityType::Group, &group_id)
        .await
        .unwrap_or(false)
    {
        return Err((StatusCode::FORBIDDEN, "Access denied"));
    }

    let repo = GroupRepository::new(&app_state.pool);
    repo.remove_member(group_id, contact_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to remove member"))?;

    Ok(Json(serde_json::json!({ "message": "Member removed" })))
}
