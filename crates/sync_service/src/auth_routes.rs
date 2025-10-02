use crate::auth::{AuthUser, LoginRequest, SignupRequest};
use crate::state::AppState;
use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};

/// POST /api/auth/signup
pub async fn signup(
    State(app_state): State<AppState>,
    Json(req): Json<SignupRequest>,
) -> impl IntoResponse {
    match app_state.auth_service.signup(req).await {
        Ok(response) => Ok(Json(response)),
        Err(err) => Err(err),
    }
}

/// POST /api/auth/login
pub async fn login(
    State(app_state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> impl IntoResponse {
    match app_state.auth_service.login(req).await {
        Ok(response) => Ok(Json(response)),
        Err(err) => Err(err),
    }
}

/// POST /api/auth/refresh
pub async fn refresh(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
) -> impl IntoResponse {
    match app_state.auth_service.refresh_token(&user.id.to_string()).await {
        Ok(token) => Ok(Json(serde_json::json!({ "token": token }))),
        Err(err) => Err(err),
    }
}

/// POST /api/auth/logout
pub async fn logout(
    AuthUser(_user): AuthUser,
) -> impl IntoResponse {
    // For JWT-based auth, logout is typically handled client-side
    // We could implement token blacklisting here if needed
    Json(serde_json::json!({ "message": "Logged out successfully" }))
}

/// GET /api/auth/me
pub async fn get_current_user(
    AuthUser(user): AuthUser,
) -> impl IntoResponse {
    Json(serde_json::json!({
        "id": user.id,
        "email": user.email,
        "name": user.name,
        "email_verified": user.email_verified,
        "active": user.active,
    }))
}