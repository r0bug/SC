use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use local_store::repositories::CommunicationRepository;
use serde::Serialize;
use sqlx::Row;
use std::io::Cursor;
use uuid::Uuid;

use crate::android_import::{
    insert_calls, insert_mms, insert_sms, parse_calls_xml, parse_mms_xml, parse_sms_xml,
};
use crate::auth::AuthUser;
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct AndroidImportResponse {
    pub file_type: String,
    pub total_records: usize,
    pub imported: usize,
    pub skipped: usize,
    pub elapsed_seconds: f64,
}

/// POST /api/import/android-calls - Import calls from Android backup XML
pub async fn import_android_calls(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let start = std::time::Instant::now();

    // Extract file from multipart
    let mut file_data = Vec::new();
    let mut file_name = String::from("calls.xml");

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("Failed to read multipart: {}", e),
        )
    })? {
        if let Some(name) = field.file_name() {
            file_name = name.to_string();
        }

        let data = field.bytes().await.map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                format!("Failed to read file: {}", e),
            )
        })?;
        file_data = data.to_vec();
    }

    if file_data.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "No file provided".to_string()));
    }

    // Parse XML
    let reader = Cursor::new(file_data);
    let calls = parse_calls_xml(reader).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("Failed to parse XML: {}", e),
        )
    })?;

    let total_records = calls.len();

    // Use authenticated user ID
    let user_id = user.id.to_string();

    // Insert into database
    let (imported, skipped) = insert_calls(&state.pool, calls, &user_id, &file_name)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to import: {}", e),
            )
        })?;

    let elapsed_seconds = start.elapsed().as_secs_f64();

    Ok(Json(AndroidImportResponse {
        file_type: "calls".to_string(),
        total_records,
        imported,
        skipped,
        elapsed_seconds,
    }))
}

/// POST /api/import/android-sms - Import SMS from Android backup XML
pub async fn import_android_sms(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let start = std::time::Instant::now();

    // Extract file from multipart
    let mut file_data = Vec::new();
    let mut file_name = String::from("sms.xml");

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("Failed to read multipart: {}", e),
        )
    })? {
        if let Some(name) = field.file_name() {
            file_name = name.to_string();
        }

        let data = field.bytes().await.map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                format!("Failed to read file: {}", e),
            )
        })?;
        file_data = data.to_vec();
    }

    if file_data.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "No file provided".to_string()));
    }

    // Detect if this is SMS or MMS XML by peeking at content
    let xml_str = String::from_utf8_lossy(&file_data);
    let is_mms = xml_str.contains("<mms") || xml_str.contains("<mmses");

    let (imported, skipped, total_records) = if is_mms {
        // Parse as MMS
        let reader = Cursor::new(file_data);
        let messages = parse_mms_xml(reader).map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                format!("Failed to parse MMS XML: {}", e),
            )
        })?;

        let total = messages.len();
        let user_id = user.id.to_string();
        let (imp, skip) = insert_mms(&state.pool, messages, &user_id, &file_name)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to import: {}", e),
                )
            })?;
        (imp, skip, total)
    } else {
        // Parse as SMS
        let reader = Cursor::new(file_data);
        let messages = parse_sms_xml(reader).map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                format!("Failed to parse SMS XML: {}", e),
            )
        })?;

        let total = messages.len();
        let user_id = user.id.to_string();
        let (imp, skip) = insert_sms(&state.pool, messages, &user_id, &file_name)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to import: {}", e),
                )
            })?;
        (imp, skip, total)
    };

    let elapsed_seconds = start.elapsed().as_secs_f64();

    Ok(Json(AndroidImportResponse {
        file_type: if is_mms {
            "mms".to_string()
        } else {
            "sms".to_string()
        },
        total_records,
        imported,
        skipped,
        elapsed_seconds,
    }))
}

/// GET /api/communications/history/:contact_id - Get communication history for a contact
pub async fn get_contact_communications(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Path(contact_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let comm_repo = CommunicationRepository::new(&state.pool);

    let contact_uuid = Uuid::parse_str(&contact_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid contact ID".to_string()))?;

    let communications = comm_repo
        .get_communications_by_contact(contact_uuid)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?;

    // Convert to JSON with proper serialization
    let comms_json: Vec<serde_json::Value> = communications
        .iter()
        .map(|comm| {
            serde_json::json!({
                "id": comm.id.to_string(),
                "contact_id": comm.contact_id.to_string(),
                "communication_type": format!("{:?}", comm.communication_type),
                "direction": format!("{:?}", comm.direction),
                "timestamp": comm.timestamp.to_rfc3339(),
                "content": comm.content,
                "duration_seconds": comm.duration_seconds,
                "phone_number": comm.phone_number,
                "thread_id": comm.thread_id,
                "status": format!("{:?}", comm.status),
                "metadata": comm.metadata,
                "created_at": comm.created_at.to_rfc3339(),
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "communications": comms_json,
        "total": comms_json.len(),
    })))
}

/// GET /api/communications/search - Search communication history
pub async fn search_communications(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let query = params.get("q").map(|s| s.as_str()).unwrap_or("");
    let limit: i64 = params
        .get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(50);

    // Search in communications table by phone_number or content
    let results = sqlx::query(
        r#"
        SELECT
            id, contact_id, communication_type, direction, timestamp,
            content, duration_seconds, phone_number, thread_id, status, metadata
        FROM communications
        WHERE phone_number LIKE ? OR content LIKE ?
        ORDER BY timestamp DESC
        LIMIT ?
        "#,
    )
    .bind(format!("%{}%", query))
    .bind(format!("%{}%", query))
    .bind(limit)
    .fetch_all(&*state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    let comms_json: Vec<serde_json::Value> = results
        .iter()
        .map(|row| {
            serde_json::json!({
                "id": row.get::<String, _>("id"),
                "contact_id": row.get::<String, _>("contact_id"),
                "communication_type": row.get::<String, _>("communication_type"),
                "direction": row.get::<String, _>("direction"),
                "timestamp": row.get::<String, _>("timestamp"),
                "content": row.get::<Option<String>, _>("content"),
                "duration_seconds": row.get::<Option<i32>, _>("duration_seconds"),
                "phone_number": row.get::<Option<String>, _>("phone_number"),
                "thread_id": row.get::<Option<String>, _>("thread_id"),
                "status": row.get::<String, _>("status"),
                "metadata": row.get::<String, _>("metadata"),
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "results": comms_json,
        "total": comms_json.len(),
        "query": query,
    })))
}
