use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use sqlx::Row;
use std::io::Cursor;

use crate::android_import::{insert_calls, insert_mms, insert_sms, parse_calls_xml, parse_mms_xml, parse_sms_xml};
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
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let start = std::time::Instant::now();

    // Extract file from multipart
    let mut file_data = Vec::new();
    let mut file_name = String::from("calls.xml");

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Failed to read multipart: {}", e)))?
    {
        if let Some(name) = field.file_name() {
            file_name = name.to_string();
        }

        let data = field
            .bytes()
            .await
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("Failed to read file: {}", e)))?;
        file_data = data.to_vec();
    }

    if file_data.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "No file provided".to_string()));
    }

    // Parse XML
    let reader = Cursor::new(file_data);
    let calls = parse_calls_xml(reader)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Failed to parse XML: {}", e)))?;

    let total_records = calls.len();

    // TODO: Get actual user_id from authentication
    let user_id = "system";

    // Insert into database
    let (imported, skipped) = insert_calls(&state.pool, calls, user_id, &file_name)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to import: {}", e)))?;

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
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let start = std::time::Instant::now();

    // Extract file from multipart
    let mut file_data = Vec::new();
    let mut file_name = String::from("sms.xml");

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Failed to read multipart: {}", e)))?
    {
        if let Some(name) = field.file_name() {
            file_name = name.to_string();
        }

        let data = field
            .bytes()
            .await
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("Failed to read file: {}", e)))?;
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
        let messages = parse_mms_xml(reader)
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("Failed to parse MMS XML: {}", e)))?;

        let total = messages.len();
        let user_id = "system";
        let (imp, skip) = insert_mms(&state.pool, messages, user_id, &file_name)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to import: {}", e)))?;
        (imp, skip, total)
    } else {
        // Parse as SMS
        let reader = Cursor::new(file_data);
        let messages = parse_sms_xml(reader)
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("Failed to parse SMS XML: {}", e)))?;

        let total = messages.len();
        let user_id = "system";
        let (imp, skip) = insert_sms(&state.pool, messages, user_id, &file_name)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to import: {}", e)))?;
        (imp, skip, total)
    };

    let elapsed_seconds = start.elapsed().as_secs_f64();

    Ok(Json(AndroidImportResponse {
        file_type: if is_mms { "mms".to_string() } else { "sms".to_string() },
        total_records,
        imported,
        skipped,
        elapsed_seconds,
    }))
}

/// GET /api/communications/history/:contact_id - Get communication history for a contact
pub async fn get_contact_communications(
    State(state): State<AppState>,
    axum::extract::Path(contact_id): axum::extract::Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    #[derive(Serialize)]
    struct CommunicationHistory {
        calls: Vec<serde_json::Value>,
        messages: Vec<serde_json::Value>,
    }

    // Get calls
    let calls = sqlx::query(
        r#"
        SELECT
            id, phone_number, contact_name, call_date, duration, call_type,
            readable_date, imported_at
        FROM call_history
        WHERE contact_id = ?
        ORDER BY call_date DESC
        LIMIT 100
        "#
    )
    .bind(&contact_id)
    .fetch_all(&*state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?;

    let calls_json: Vec<serde_json::Value> = calls
        .iter()
        .map(|row| {
            serde_json::json!({
                "id": row.get::<String, _>("id"),
                "phone_number": row.get::<String, _>("phone_number"),
                "contact_name": row.get::<Option<String>, _>("contact_name"),
                "call_date": row.get::<i64, _>("call_date"),
                "duration": row.get::<i64, _>("duration"),
                "call_type": row.get::<i32, _>("call_type"),
                "readable_date": row.get::<String, _>("readable_date"),
                "imported_at": row.get::<String, _>("imported_at"),
            })
        })
        .collect();

    // Get messages
    let messages = sqlx::query(
        r#"
        SELECT
            id, phone_number, contact_name, message_date, message_type,
            subject, body, readable_date, imported_at
        FROM sms_history
        WHERE contact_id = ?
        ORDER BY message_date DESC
        LIMIT 100
        "#
    )
    .bind(&contact_id)
    .fetch_all(&*state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?;

    let messages_json: Vec<serde_json::Value> = messages
        .iter()
        .map(|row| {
            serde_json::json!({
                "id": row.get::<String, _>("id"),
                "phone_number": row.get::<String, _>("phone_number"),
                "contact_name": row.get::<Option<String>, _>("contact_name"),
                "message_date": row.get::<i64, _>("message_date"),
                "message_type": row.get::<i32, _>("message_type"),
                "subject": row.get::<Option<String>, _>("subject"),
                "body": row.get::<String, _>("body"),
                "readable_date": row.get::<String, _>("readable_date"),
                "imported_at": row.get::<String, _>("imported_at"),
            })
        })
        .collect();

    Ok(Json(CommunicationHistory {
        calls: calls_json,
        messages: messages_json,
    }))
}

/// GET /api/communications/search - Search communication history
pub async fn search_communications(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let query = params.get("q").map(|s| s.as_str()).unwrap_or("");
    let limit: i64 = params
        .get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(50);

    #[derive(Serialize)]
    struct SearchResults {
        calls: Vec<serde_json::Value>,
        messages: Vec<serde_json::Value>,
    }

    // Search in calls (by contact_name or phone_number)
    let calls = sqlx::query(
        r#"
        SELECT
            id, phone_number, contact_name, call_date, duration, call_type,
            readable_date, contact_id
        FROM call_history
        WHERE contact_name LIKE ? OR phone_number LIKE ?
        ORDER BY call_date DESC
        LIMIT ?
        "#
    )
    .bind(format!("%{}%", query))
    .bind(format!("%{}%", query))
    .bind(limit)
    .fetch_all(&*state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?;

    let calls_json: Vec<serde_json::Value> = calls
        .iter()
        .map(|row| {
            serde_json::json!({
                "id": row.get::<String, _>("id"),
                "phone_number": row.get::<String, _>("phone_number"),
                "contact_name": row.get::<Option<String>, _>("contact_name"),
                "call_date": row.get::<i64, _>("call_date"),
                "duration": row.get::<i64, _>("duration"),
                "call_type": row.get::<i32, _>("call_type"),
                "readable_date": row.get::<String, _>("readable_date"),
                "contact_id": row.get::<Option<String>, _>("contact_id"),
            })
        })
        .collect();

    // Search in messages (by contact_name, phone_number, or body text)
    let messages = sqlx::query(
        r#"
        SELECT
            id, phone_number, contact_name, message_date, message_type,
            subject, body, readable_date, contact_id
        FROM sms_history
        WHERE contact_name LIKE ? OR phone_number LIKE ? OR body LIKE ?
        ORDER BY message_date DESC
        LIMIT ?
        "#
    )
    .bind(format!("%{}%", query))
    .bind(format!("%{}%", query))
    .bind(format!("%{}%", query))
    .bind(limit)
    .fetch_all(&*state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?;

    let messages_json: Vec<serde_json::Value> = messages
        .iter()
        .map(|row| {
            serde_json::json!({
                "id": row.get::<String, _>("id"),
                "phone_number": row.get::<String, _>("phone_number"),
                "contact_name": row.get::<Option<String>, _>("contact_name"),
                "message_date": row.get::<i64, _>("message_date"),
                "message_type": row.get::<i32, _>("message_type"),
                "subject": row.get::<Option<String>, _>("subject"),
                "body": row.get::<String, _>("body"),
                "readable_date": row.get::<String, _>("readable_date"),
                "contact_id": row.get::<Option<String>, _>("contact_id"),
            })
        })
        .collect();

    Ok(Json(SearchResults {
        calls: calls_json,
        messages: messages_json,
    }))
}
