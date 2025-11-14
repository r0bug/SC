use crate::audit;
use crate::auth::{AuthError, AuthUser, LoginRequest, SignupRequest};
use crate::state::AppState;
use axum::{extract::State, http::HeaderMap, response::IntoResponse, Json};

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
            let _ = app_state.audit_service.log_signup(user_id, ip, user_agent).await;
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
            // TODO: Log failed login attempt (would need user_id from email lookup)
            // For now, failed attempts are not logged due to lack of user_id
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
    }))
}
