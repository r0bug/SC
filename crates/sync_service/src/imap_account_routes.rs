use crate::{auth::AuthUser, state::AppState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post, put},
    Json, Router,
};
use chrono::Utc;
use local_store::repositories::{ImapAccount, ImapAccountRepository};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// --- Request / Response types ---

#[derive(Debug, Deserialize)]
pub struct CreateImapAccountRequest {
    pub name: String,
    pub server: String,
    pub port: Option<i32>,
    pub username: String,
    pub password: String,
    pub default_folder: Option<String>,
    pub max_emails: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateImapAccountRequest {
    pub name: Option<String>,
    pub server: Option<String>,
    pub port: Option<i32>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub default_folder: Option<String>,
    pub max_emails: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct FetchOverride {
    pub folder: Option<String>,
    pub max_emails: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct ImapAccountResponse {
    pub id: String,
    pub name: String,
    pub server: String,
    pub port: i32,
    pub username: String,
    pub default_folder: String,
    pub max_emails: i32,
    pub last_fetched_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

fn to_response(a: &ImapAccount) -> ImapAccountResponse {
    ImapAccountResponse {
        id: a.id.to_string(),
        name: a.name.clone(),
        server: a.server.clone(),
        port: a.port,
        username: a.username.clone(),
        default_folder: a.default_folder.clone(),
        max_emails: a.max_emails,
        last_fetched_at: a.last_fetched_at.map(|t| t.to_rfc3339()),
        created_at: a.created_at.to_rfc3339(),
        updated_at: a.updated_at.to_rfc3339(),
    }
}

// --- Router ---

pub fn imap_account_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/imap-accounts",
            get(list_accounts).post(create_account),
        )
        .route(
            "/api/imap-accounts/:id",
            put(update_account).delete(delete_account),
        )
        .route("/api/imap-accounts/:id/fetch", post(fetch_from_account))
}

// --- Handlers ---

/// GET /api/imap-accounts
async fn list_accounts(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let repo = ImapAccountRepository::new(&app_state.pool);

    let accounts = repo
        .list_by_user(user.id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let response: Vec<ImapAccountResponse> = accounts.iter().map(to_response).collect();
    Ok(Json(serde_json::json!({ "accounts": response })))
}

/// POST /api/imap-accounts
async fn create_account(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(req): Json<CreateImapAccountRequest>,
) -> Result<Json<ImapAccountResponse>, (StatusCode, String)> {
    let name = req.name.trim().to_string();
    if name.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Account name is required".to_string()));
    }
    if req.server.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Server is required".to_string()));
    }
    if req.username.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Username is required".to_string()));
    }

    let now = Utc::now();
    let account = ImapAccount {
        id: Uuid::new_v4(),
        user_id: user.id,
        name,
        server: req.server.trim().to_string(),
        port: req.port.unwrap_or(993),
        username: req.username.trim().to_string(),
        password: req.password,
        default_folder: req.default_folder.unwrap_or_else(|| "INBOX".to_string()),
        max_emails: req.max_emails.unwrap_or(500),
        last_fetched_at: None,
        created_at: now,
        updated_at: now,
    };

    let repo = ImapAccountRepository::new(&app_state.pool);
    repo.create(&account)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(to_response(&account)))
}

/// PUT /api/imap-accounts/:id
async fn update_account(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateImapAccountRequest>,
) -> Result<Json<ImapAccountResponse>, (StatusCode, String)> {
    let repo = ImapAccountRepository::new(&app_state.pool);

    let mut account = repo
        .get_by_id(id)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

    // Verify ownership
    if account.user_id != user.id {
        return Err((StatusCode::FORBIDDEN, "Not your account".to_string()));
    }

    if let Some(name) = req.name {
        account.name = name.trim().to_string();
    }
    if let Some(server) = req.server {
        account.server = server.trim().to_string();
    }
    if let Some(port) = req.port {
        account.port = port;
    }
    if let Some(username) = req.username {
        account.username = username.trim().to_string();
    }
    if let Some(password) = req.password {
        account.password = password;
    }
    if let Some(folder) = req.default_folder {
        account.default_folder = folder;
    }
    if let Some(max) = req.max_emails {
        account.max_emails = max;
    }
    account.updated_at = Utc::now();

    repo.update(&account)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(to_response(&account)))
}

/// DELETE /api/imap-accounts/:id
async fn delete_account(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let repo = ImapAccountRepository::new(&app_state.pool);

    let account = repo
        .get_by_id(id)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

    if account.user_id != user.id {
        return Err((StatusCode::FORBIDDEN, "Not your account".to_string()));
    }

    repo.delete(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/imap-accounts/:id/fetch
async fn fetch_from_account(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
    body: Option<Json<FetchOverride>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let repo = ImapAccountRepository::new(&app_state.pool);

    let account = repo
        .get_by_id(id)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

    if account.user_id != user.id {
        return Err((StatusCode::FORBIDDEN, "Not your account".to_string()));
    }

    let overrides = body.map(|j| j.0);
    let folder = overrides
        .as_ref()
        .and_then(|o| o.folder.clone())
        .unwrap_or_else(|| account.default_folder.clone());
    let max_emails = overrides
        .as_ref()
        .and_then(|o| o.max_emails)
        .unwrap_or(account.max_emails as u32);

    let (fetched, stored, skipped) = crate::email_import::fetch_emails(
        &account.server,
        account.port as u16,
        &account.username,
        &account.password,
        &folder,
        max_emails,
        &app_state.pool,
        Some(user.id),
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Fetch failed: {}", e)))?;

    // Update last_fetched_at
    let now = Utc::now();
    let _ = repo.update_last_fetched(id, now).await;

    Ok(Json(serde_json::json!({
        "fetched": fetched,
        "stored": stored,
        "skipped": skipped,
        "server": account.server,
        "folder": folder,
    })))
}
