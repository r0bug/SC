use crate::{auth::AuthUser, state::AppState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use core_domain::{Permission, ShareEntityType, ShareInvite};
use local_store::repositories::{ShareRepository, UserRepository};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateShareRequest {
    pub entity_type: String,
    pub entity_id: Uuid,
    #[serde(alias = "shared_with_email")]
    pub email: String,
    pub permissions: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ShareResponse {
    pub id: Uuid,
    pub message: String,
}

/// POST /api/shares
pub async fn create_share(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(req): Json<CreateShareRequest>,
) -> impl IntoResponse {
    // Normalize entity_type to lowercase for case-insensitive matching
    let entity_type = match req.entity_type.to_lowercase().as_str() {
        "contact" => ShareEntityType::Contact,
        "project" => ShareEntityType::Project,
        "note" => ShareEntityType::Note,
        "calendar_event" | "calendarevent" => ShareEntityType::CalendarEvent,
        "group" => ShareEntityType::Group,
        _ => return Err((StatusCode::BAD_REQUEST, "Invalid entity type")),
    };

    // Check if user can share this entity
    if !app_state
        .acl_service
        .can_share(&user.id, entity_type, &req.entity_id)
        .await
        .unwrap_or(false)
    {
        return Err((StatusCode::FORBIDDEN, "You cannot share this entity"));
    }

    // Parse permissions (case-insensitive)
    let permissions: Vec<Permission> = req
        .permissions
        .iter()
        .filter_map(|p| match p.to_lowercase().as_str() {
            "read" => Some(Permission::Read),
            "write" => Some(Permission::Write),
            "share" => Some(Permission::Share),
            "delete" => Some(Permission::Delete),
            _ => None,
        })
        .collect();

    if permissions.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "No valid permissions specified"));
    }

    // Check if target user exists
    let user_repo = UserRepository::new(&app_state.pool);
    let target_user = user_repo.get_by_email(&req.email).await.ok();

    // Create share invite
    let invite = ShareInvite {
        id: Uuid::new_v4(),
        entity_type,
        entity_id: req.entity_id,
        shared_by: user.id,
        shared_with_email: req.email.clone(),
        shared_with_user: target_user.as_ref().map(|u| u.id),
        permissions,
        accepted: false,
        accepted_at: None,
        revoked: false,
        revoked_at: None,
        created_at: chrono::Utc::now(),
        expires_at: Some(chrono::Utc::now() + chrono::Duration::days(7)),
    };

    let share_repo = ShareRepository::new(&app_state.pool);
    share_repo
        .create(&invite)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create share"))?;

    Ok(Json(ShareResponse {
        id: invite.id,
        message: format!("Share invite sent to {}", req.email),
    }))
}

/// POST /api/shares/:id/accept
pub async fn accept_share(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(share_id): Path<Uuid>,
) -> impl IntoResponse {
    let share_repo = ShareRepository::new(&app_state.pool);

    // Get the share invite
    let mut invite = share_repo
        .get_by_id(share_id)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Share invite not found"))?;

    // Check if the invite is for this user
    if invite.shared_with_email != user.email && invite.shared_with_user != Some(user.id) {
        return Err((StatusCode::FORBIDDEN, "This share invite is not for you"));
    }

    // Check if already accepted or revoked
    if invite.accepted {
        return Err((StatusCode::BAD_REQUEST, "Share already accepted"));
    }
    if invite.revoked {
        return Err((StatusCode::BAD_REQUEST, "Share has been revoked"));
    }

    // Check if expired
    if let Some(expires_at) = invite.expires_at {
        if expires_at < chrono::Utc::now() {
            return Err((StatusCode::BAD_REQUEST, "Share invite has expired"));
        }
    }

    // Accept the share
    invite.accepted = true;
    invite.accepted_at = Some(chrono::Utc::now());
    invite.shared_with_user = Some(user.id);

    share_repo
        .update(&invite)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to accept share"))?;

    // Grant permissions via ACL
    app_state
        .acl_service
        .grant_permission(
            &invite.shared_by,
            &user.id,
            invite.entity_type,
            &invite.entity_id,
            invite.permissions,
        )
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to grant permissions",
            )
        })?;

    Ok(Json(serde_json::json!({
        "message": "Share accepted successfully",
        "entity_id": invite.entity_id,
    })))
}

/// POST /api/shares/:id/reject
pub async fn reject_share(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(share_id): Path<Uuid>,
) -> impl IntoResponse {
    let share_repo = ShareRepository::new(&app_state.pool);

    // Get the share invite
    let mut invite = share_repo
        .get_by_id(share_id)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Share invite not found"))?;

    // Check if the invite is for this user
    if invite.shared_with_email != user.email && invite.shared_with_user != Some(user.id) {
        return Err((StatusCode::FORBIDDEN, "This share invite is not for you"));
    }

    // Check if already accepted or revoked
    if invite.accepted {
        return Err((StatusCode::BAD_REQUEST, "Share already accepted"));
    }
    if invite.revoked {
        return Err((StatusCode::BAD_REQUEST, "Share has been revoked"));
    }

    // Revoke the share (rejection is treated as revocation by recipient)
    invite.revoked = true;
    invite.revoked_at = Some(chrono::Utc::now());

    share_repo
        .update(&invite)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to reject share"))?;

    Ok(Json(serde_json::json!({
        "message": "Share rejected successfully",
    })))
}

/// POST /api/shares/:id/revoke
pub async fn revoke_share(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(share_id): Path<Uuid>,
) -> impl IntoResponse {
    let share_repo = ShareRepository::new(&app_state.pool);

    // Get the share invite
    let mut invite = share_repo
        .get_by_id(share_id)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Share invite not found"))?;

    // Check if user is the one who shared
    if invite.shared_by != user.id {
        return Err((StatusCode::FORBIDDEN, "You cannot revoke this share"));
    }

    // Check if already revoked
    if invite.revoked {
        return Err((StatusCode::BAD_REQUEST, "Share already revoked"));
    }

    // Revoke the share
    invite.revoked = true;
    invite.revoked_at = Some(chrono::Utc::now());

    share_repo
        .update(&invite)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to revoke share"))?;

    // If share was accepted, revoke permissions
    if invite.accepted {
        if let Some(shared_with_user) = invite.shared_with_user {
            app_state
                .acl_service
                .revoke_permission(
                    &user.id,
                    &shared_with_user,
                    invite.entity_type,
                    &invite.entity_id,
                )
                .await
                .map_err(|_| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to revoke permissions",
                    )
                })?;
        }
    }

    Ok(Json(serde_json::json!({
        "message": "Share revoked successfully",
    })))
}

/// GET /api/shares/sent
pub async fn list_sent_shares(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<Vec<ShareInvite>>, (StatusCode, &'static str)> {
    let share_repo = ShareRepository::new(&app_state.pool);

    let shares = share_repo
        .list_by_sharer(user.id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to list shares"))?;

    Ok(Json(shares))
}

/// GET /api/shares/received
pub async fn list_received_shares(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<Vec<ShareInvite>>, (StatusCode, &'static str)> {
    let share_repo = ShareRepository::new(&app_state.pool);

    let shares = share_repo
        .list_by_recipient(&user.email)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to list shares"))?;

    Ok(Json(shares))
}
