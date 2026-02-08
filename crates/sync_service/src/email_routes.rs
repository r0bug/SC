use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use local_store::repositories::EmailHistoryRepository;
use serde::{Deserialize, Serialize};
use sqlx::Row;

use crate::auth::AuthUser;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct FetchEmailsRequest {
    pub server: String,
    pub port: Option<u16>,
    pub username: String,
    pub password: String,
    pub folder: Option<String>,
    pub max_emails: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct FetchEmailsResponse {
    pub fetched: usize,
    pub stored: usize,
    pub skipped: usize,
    pub server: String,
    pub folder: String,
}

#[derive(Debug, Serialize)]
pub struct EmailStatsResponse {
    pub total_emails: i64,
    pub unique_senders: i64,
    pub earliest_message_date: Option<i64>,
    pub latest_message_date: Option<i64>,
    pub received_count: i64,
    pub sent_count: i64,
}

/// POST /api/email/fetch - Trigger IMAP email fetch
pub async fn fetch_emails(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Json(req): Json<FetchEmailsRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let port = req.port.unwrap_or(993);
    let folder = req.folder.unwrap_or_else(|| "INBOX".to_string());
    let max_emails = req.max_emails.unwrap_or(100);

    let (fetched, stored, skipped) =
        crate::email_import::fetch_emails(crate::email_import::FetchEmailsConfig {
            server: &req.server,
            port,
            username: &req.username,
            password: &req.password,
            folder: &folder,
            max_emails,
            pool: &state.pool,
            user_id: Some(user.id),
        })
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Email fetch failed: {}", e),
            )
        })?;

    Ok(Json(FetchEmailsResponse {
        fetched,
        stored,
        skipped,
        server: req.server,
        folder,
    }))
}

/// GET /api/email/conversations - List email conversations grouped by sender
pub async fn list_email_conversations(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let rows = sqlx::query(
        r#"
        SELECT
            from_address,
            from_name,
            contact_id,
            COUNT(*) as message_count,
            MAX(message_date) as last_message_date
        FROM email_history
        WHERE message_type = 1
        GROUP BY from_address
        ORDER BY last_message_date DESC
        "#,
    )
    .fetch_all(&*state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    let mut conversations: Vec<serde_json::Value> = Vec::new();
    for row in &rows {
        let addr: String = row.get("from_address");

        // Get subject preview from most recent email
        let preview_row = sqlx::query(
            "SELECT subject, body_text FROM email_history WHERE from_address = ? ORDER BY message_date DESC LIMIT 1",
        )
        .bind(&addr)
        .fetch_optional(&*state.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?;

        let (subject_preview, body_preview) = preview_row
            .map(|r| {
                let subj: String = r.get("subject");
                let body: Option<String> = r.get("body_text");
                let body_short = body
                    .map(|b| {
                        if b.len() > 100 {
                            let mut end = 100;
                            while !b.is_char_boundary(end) {
                                end -= 1;
                            }
                            format!("{}...", &b[..end])
                        } else {
                            b
                        }
                    })
                    .unwrap_or_default();
                (subj, body_short)
            })
            .unwrap_or_default();

        conversations.push(serde_json::json!({
            "from_address": addr,
            "from_name": row.get::<Option<String>, _>("from_name"),
            "contact_id": row.get::<Option<String>, _>("contact_id"),
            "message_count": row.get::<i64, _>("message_count"),
            "last_message_date": row.get::<i64, _>("last_message_date"),
            "last_subject": subject_preview,
            "last_body_preview": body_preview,
        }));
    }

    Ok(Json(serde_json::json!({
        "conversations": conversations,
        "total": conversations.len(),
    })))
}

/// GET /api/email/conversation/:address - Get all emails for an address
pub async fn get_email_conversation(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let repo = EmailHistoryRepository::new(&state.pool);

    let messages = repo.get_by_address(&address).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    let messages_json: Vec<serde_json::Value> = messages
        .iter()
        .map(|msg| {
            serde_json::json!({
                "id": msg.id.to_string(),
                "contact_id": msg.contact_id.map(|id| id.to_string()),
                "from_address": msg.from_address,
                "from_name": msg.from_name,
                "to_address": msg.to_address,
                "cc": msg.cc,
                "subject": msg.subject,
                "body_text": msg.body_text,
                "body_html": msg.body_html,
                "message_date": msg.message_date,
                "message_type": msg.message_type,
                "message_id": msg.message_id,
                "read_status": msg.read_status,
                "has_attachments": msg.has_attachments,
                "folder": msg.folder,
                "imported_at": msg.imported_at.to_rfc3339(),
                "source_server": msg.source_server,
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "messages": messages_json,
        "total": messages_json.len(),
        "address": address,
    })))
}

/// GET /api/email/stats - Get email statistics
pub async fn get_email_stats(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let stats_row = sqlx::query(
        r#"
        SELECT
            COUNT(*) as total_emails,
            COUNT(DISTINCT from_address) as unique_senders,
            MIN(message_date) as earliest_message_date,
            MAX(message_date) as latest_message_date,
            SUM(CASE WHEN message_type = 1 THEN 1 ELSE 0 END) as received_count,
            SUM(CASE WHEN message_type = 2 THEN 1 ELSE 0 END) as sent_count
        FROM email_history
        "#,
    )
    .fetch_one(&*state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    Ok(Json(EmailStatsResponse {
        total_emails: stats_row.get("total_emails"),
        unique_senders: stats_row.get("unique_senders"),
        earliest_message_date: stats_row.get("earliest_message_date"),
        latest_message_date: stats_row.get("latest_message_date"),
        received_count: stats_row.get("received_count"),
        sent_count: stats_row.get("sent_count"),
    }))
}

/// GET /api/email/search?q=... - Search emails
pub async fn search_emails(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let query = params.get("q").map(|s| s.as_str()).unwrap_or("");
    let limit: i64 = params
        .get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(50);

    if query.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Query parameter 'q' is required".to_string(),
        ));
    }

    let repo = EmailHistoryRepository::new(&state.pool);

    let messages = repo.search(query, limit).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    let results: Vec<serde_json::Value> = messages
        .iter()
        .map(|msg| {
            serde_json::json!({
                "id": msg.id.to_string(),
                "from_address": msg.from_address,
                "from_name": msg.from_name,
                "to_address": msg.to_address,
                "subject": msg.subject,
                "body_preview": msg.body_text.as_ref().map(|b| {
                    if b.len() > 200 { let mut e = 200; while !b.is_char_boundary(e) { e -= 1; } format!("{}...", &b[..e]) } else { b.clone() }
                }),
                "message_date": msg.message_date,
                "message_type": msg.message_type,
                "folder": msg.folder,
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "results": results,
        "total": results.len(),
        "query": query,
    })))
}

/// Build email routes as a Router
pub fn email_routes() -> Router<AppState> {
    Router::new()
        .route("/api/email/fetch", axum::routing::post(fetch_emails))
        .route("/api/email/conversations", get(list_email_conversations))
        .route(
            "/api/email/conversation/:address",
            get(get_email_conversation),
        )
        .route("/api/email/stats", get(get_email_stats))
        .route("/api/email/search", get(search_emails))
}
