use crate::audit;
use crate::auth::{AuthError, AuthUser, LoginRequest, SignupRequest};
use crate::state::AppState;
use axum::{extract::State, http::HeaderMap, response::IntoResponse, Json};
use local_store::repositories::UserRepository;

/// POST /api/auth/signup
pub async fn signup(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<SignupRequest>,
) -> impl IntoResponse {
    let ip = audit::extract_ip_address(&headers);
    let user_agent = audit::extract_user_agent(&headers);

    match app_state.auth_service.signup(req).await {
        Ok(response) => {
            // Log successful signup
            let user_id = response.user.id;
            let _ = app_state
                .audit_service
                .log_signup(user_id, ip, user_agent)
                .await;
            Ok(Json(response))
        }
        Err(err) => Err(err),
    }
}

/// POST /api/auth/login
pub async fn login(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<LoginRequest>,
) -> impl IntoResponse {
    let ip = audit::extract_ip_address(&headers);
    let user_agent = audit::extract_user_agent(&headers);
    let email = req.email.clone();

    match app_state.auth_service.login(req).await {
        Ok(response) => {
            // Log successful login
            let user_id = response.user.id;
            let _ = app_state
                .audit_service
                .log_login(user_id, true, ip, user_agent)
                .await;
            Ok(Json(response))
        }
        Err(err) => {
            // Log failed login attempt
            // Try to find the user by email to get their user_id for the audit log
            let user_repo = UserRepository::new(&app_state.pool);
            if let Ok(user) = user_repo.get_by_email(&email).await {
                let _ = app_state
                    .audit_service
                    .log_login(user.id, false, ip.clone(), user_agent.clone())
                    .await;
            }
            // Even if user doesn't exist, we've attempted logging
            Err(err)
        }
    }
}

/// POST /api/auth/refresh
/// NOTE: We need the original JWT token, not just the user, to refresh it
pub async fn refresh(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    AuthUser(_user): AuthUser, // Still validate auth, but don't use the user
) -> impl IntoResponse {
    // Extract the token from Authorization header
    let auth_header = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .ok_or(AuthError::InvalidToken)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(AuthError::InvalidToken)?;

    // Refresh the token (validates and generates new one)
    match app_state.auth_service.refresh_token(token).await {
        Ok(new_token) => Ok(Json(serde_json::json!({ "token": new_token }))),
        Err(err) => Err(err),
    }
}

/// POST /api/auth/logout
pub async fn logout(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    AuthUser(user): AuthUser,
) -> impl IntoResponse {
    let ip = audit::extract_ip_address(&headers);
    let user_agent = audit::extract_user_agent(&headers);

    // Log logout event
    let _ = app_state
        .audit_service
        .log_logout(user.id, ip, user_agent)
        .await;

    // For JWT-based auth, logout is typically handled client-side
    // We could implement token blacklisting here if needed
    Json(serde_json::json!({ "message": "Logged out successfully" }))
}

/// GET /api/auth/me
pub async fn get_current_user(AuthUser(user): AuthUser) -> impl IntoResponse {
    Json(serde_json::json!({
        "id": user.id,
        "email": user.email,
        "name": user.name,
        "email_verified": user.email_verified,
        "active": user.active,
        "preferences": user.preferences,
        "created_at": user.created_at,
        "updated_at": user.updated_at,
        "last_login_at": user.last_login_at,
    }))
}

/// PUT /api/auth/me
#[derive(serde::Deserialize)]
pub struct UpdateProfileRequest {
    pub name: Option<String>,
    pub email: Option<String>,
    pub preferences: Option<serde_json::Value>,
}

pub async fn update_current_user(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    AuthUser(user): AuthUser,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<impl IntoResponse, AuthError> {
    let user_repo = local_store::repositories::UserRepository::new(&app_state.pool);
    let mut updated_user = user.clone();
    let mut changes = serde_json::Map::new();

    if let Some(name) = req.name {
        crate::validation::validate_name(&name).map_err(|_| AuthError::InvalidCredentials)?;
        changes.insert("name".to_string(), serde_json::json!(name));
        updated_user.name = name;
    }

    if let Some(email) = req.email {
        crate::validation::validate_email(&email).map_err(|_| AuthError::InvalidCredentials)?;
        if let Ok(existing) = user_repo.get_by_email(&email).await {
            if existing.id != user.id {
                return Err(AuthError::UserAlreadyExists);
            }
        }
        changes.insert("email".to_string(), serde_json::json!(email));
        updated_user.email = email;
    }

    if let Some(preferences) = req.preferences {
        changes.insert("preferences".to_string(), serde_json::json!("updated"));
        updated_user.preferences = preferences;
    }

    updated_user.updated_at = chrono::Utc::now();
    user_repo
        .update(&updated_user)
        .await
        .map_err(|_| AuthError::InternalError)?;

    // Audit log: profile updated
    let ip = audit::extract_ip_address(&headers);
    let user_agent = audit::extract_user_agent(&headers);
    let _ = app_state
        .audit_service
        .log_operation(
            core_domain::ShareEntityType::Contact, // Using Contact as placeholder for user events
            user.id,
            core_domain::AuditAction::Update,
            user.id,
            serde_json::json!({"event": "profile_updated", "changes": changes}),
            ip,
            user_agent,
        )
        .await;

    Ok(Json(serde_json::json!({
        "id": updated_user.id,
        "email": updated_user.email,
        "name": updated_user.name,
        "email_verified": updated_user.email_verified,
        "active": updated_user.active,
        "preferences": updated_user.preferences,
        "created_at": updated_user.created_at,
        "updated_at": updated_user.updated_at,
        "last_login_at": updated_user.last_login_at,
    })))
}

/// POST /api/auth/change-password
#[derive(serde::Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

pub async fn change_password(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    AuthUser(user): AuthUser,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<impl IntoResponse, AuthError> {
    app_state
        .auth_service
        .change_password(user.id, &req.current_password, &req.new_password)
        .await?;

    // Audit log: password changed
    let ip = audit::extract_ip_address(&headers);
    let user_agent = audit::extract_user_agent(&headers);
    let _ = app_state
        .audit_service
        .log_operation(
            core_domain::ShareEntityType::Contact, // Using Contact as placeholder for user events
            user.id,
            core_domain::AuditAction::Update,
            user.id,
            serde_json::json!({"event": "password_changed"}),
            ip,
            user_agent,
        )
        .await;

    Ok(Json(serde_json::json!({
        "message": "Password changed successfully"
    })))
}

/// POST /api/auth/logout-all
pub async fn logout_all(AuthUser(_user): AuthUser) -> impl IntoResponse {
    Json(serde_json::json!({
        "message": "Logged out of all sessions (session revocation coming soon)"
    }))
}
