use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use local_store::repositories::{CommunicationRepository, SmsHistoryRepository};
use serde::{Deserialize, Serialize};
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

    // Auto-link imported calls to contacts
    crate::contact_reconciliation::auto_link_after_sms_import(&state.pool, user.id).await;

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

    // Auto-link imported SMS/MMS to contacts
    crate::contact_reconciliation::auto_link_after_sms_import(&state.pool, user.id).await;

    // Run concept detection on newly imported SMS (scan all concepts with matchers)
    if imported > 0 {
        let detector = crate::concept_detection::ConceptDetector::new(&state.pool);
        if let Err(e) = detector.scan_all().await {
            tracing::warn!("Post-import SMS concept detection failed: {}", e);
        }
    }

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

// =====================================================
// SMS History endpoints (sms_history table)
// =====================================================

#[derive(Debug, Serialize)]
pub struct SmsStatsResponse {
    pub total_messages: i64,
    pub unique_conversations: i64,
    pub earliest_message_date: Option<i64>,
    pub latest_message_date: Option<i64>,
    pub messages_with_contacts: i64,
    pub messages_without_contacts: i64,
}

/// GET /api/sms-history/conversations - List unique conversations
pub async fn list_sms_conversations(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let rows = sqlx::query(
        r#"
        SELECT
            phone_number,
            contact_name,
            contact_id,
            COUNT(*) as message_count,
            MAX(message_date) as last_message_date
        FROM sms_history
        GROUP BY phone_number
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
        let phone: String = row.get("phone_number");
        let last_date: i64 = row.get("last_message_date");

        // Get the most recent message body for preview
        let preview_row = sqlx::query(
            "SELECT body FROM sms_history WHERE phone_number = ? ORDER BY message_date DESC LIMIT 1",
        )
        .bind(&phone)
        .fetch_optional(&*state.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?;

        let preview = preview_row
            .map(|r| {
                let body: String = r.get("body");
                if body.len() > 100 {
                    let mut e = 100;
                    while !body.is_char_boundary(e) {
                        e -= 1;
                    }
                    format!("{}...", &body[..e])
                } else {
                    body
                }
            })
            .unwrap_or_default();

        conversations.push(serde_json::json!({
            "phone_number": phone,
            "contact_name": row.get::<Option<String>, _>("contact_name"),
            "contact_id": row.get::<Option<String>, _>("contact_id"),
            "message_count": row.get::<i64, _>("message_count"),
            "last_message_date": last_date,
            "last_message_preview": preview,
        }));
    }

    Ok(Json(serde_json::json!({
        "conversations": conversations,
        "total": conversations.len(),
    })))
}

/// GET /api/sms-history/conversation/:phone - Get all messages for a phone number
pub async fn get_sms_conversation(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Path(phone): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let sms_repo = SmsHistoryRepository::new(&state.pool);

    let messages = sms_repo.get_by_phone(&phone).await.map_err(|e| {
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
                "phone_number": msg.phone_number,
                "contact_name": msg.contact_name,
                "message_date": msg.message_date,
                "message_type": msg.message_type,
                "subject": msg.subject,
                "body": msg.body,
                "readable_date": msg.readable_date,
                "thread_id": msg.thread_id,
                "read_status": msg.read_status,
                "imported_at": msg.imported_at.to_rfc3339(),
                "source_file": msg.source_file,
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "messages": messages_json,
        "total": messages_json.len(),
        "phone_number": phone,
    })))
}

/// GET /api/sms-history/stats - Get import statistics
pub async fn get_sms_stats(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let stats_row = sqlx::query(
        r#"
        SELECT
            COUNT(*) as total_messages,
            COUNT(DISTINCT phone_number) as unique_conversations,
            MIN(message_date) as earliest_message_date,
            MAX(message_date) as latest_message_date,
            SUM(CASE WHEN contact_id IS NOT NULL THEN 1 ELSE 0 END) as messages_with_contacts,
            SUM(CASE WHEN contact_id IS NULL THEN 1 ELSE 0 END) as messages_without_contacts
        FROM sms_history
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

    Ok(Json(SmsStatsResponse {
        total_messages: stats_row.get("total_messages"),
        unique_conversations: stats_row.get("unique_conversations"),
        earliest_message_date: stats_row.get("earliest_message_date"),
        latest_message_date: stats_row.get("latest_message_date"),
        messages_with_contacts: stats_row.get("messages_with_contacts"),
        messages_without_contacts: stats_row.get("messages_without_contacts"),
    }))
}

/// DELETE /api/sms-history/source/:filename - Delete all messages from a specific import file
pub async fn delete_sms_by_source(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Path(filename): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let result = sqlx::query("DELETE FROM sms_history WHERE source_file = ?")
        .bind(&filename)
        .execute(&*state.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?;

    Ok(Json(serde_json::json!({
        "deleted": result.rows_affected(),
        "source_file": filename,
    })))
}

// =====================================================
// Server-side directory import
// =====================================================

#[derive(Debug, Deserialize)]
pub struct DirectoryImportRequest {
    pub path: String,
}

#[derive(Debug, Serialize)]
pub struct DirectoryImportFileResult {
    pub file_name: String,
    pub file_type: String,
    pub total_records: usize,
    pub imported: usize,
    pub skipped: usize,
    pub status: String,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DirectoryImportResponse {
    pub path: String,
    pub total_files: usize,
    pub files_imported: usize,
    pub files_failed: usize,
    pub total_records: usize,
    pub total_imported: usize,
    pub total_skipped: usize,
    pub elapsed_seconds: f64,
    pub results: Vec<DirectoryImportFileResult>,
}

/// POST /api/import/android-sms/directory - Import all XML files from a server directory
pub async fn import_directory(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Json(body): Json<DirectoryImportRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let start = std::time::Instant::now();
    let dir_path = std::path::Path::new(&body.path);

    if !dir_path.exists() {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Directory does not exist: {}", body.path),
        ));
    }
    if !dir_path.is_dir() {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Path is not a directory: {}", body.path),
        ));
    }

    // Collect all .xml files in the directory
    let mut xml_files: Vec<std::path::PathBuf> = Vec::new();
    let entries = std::fs::read_dir(dir_path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to read directory: {}", e),
        )
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to read directory entry: {}", e),
            )
        })?;
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext.eq_ignore_ascii_case("xml") {
                    xml_files.push(path);
                }
            }
        }
    }

    xml_files.sort();

    if xml_files.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("No XML files found in: {}", body.path),
        ));
    }

    let user_id = user.id.to_string();
    let mut results: Vec<DirectoryImportFileResult> = Vec::new();
    let mut total_imported = 0usize;
    let mut total_skipped = 0usize;
    let mut total_records = 0usize;
    let mut files_imported = 0usize;
    let mut files_failed = 0usize;

    for xml_path in &xml_files {
        let file_name = xml_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown.xml".to_string());

        // Read file contents
        let file_data = match std::fs::read(xml_path) {
            Ok(data) => data,
            Err(e) => {
                files_failed += 1;
                results.push(DirectoryImportFileResult {
                    file_name,
                    file_type: "unknown".to_string(),
                    total_records: 0,
                    imported: 0,
                    skipped: 0,
                    status: "error".to_string(),
                    error: Some(format!("Failed to read file: {}", e)),
                });
                continue;
            }
        };

        // Detect SMS vs MMS vs Calls
        let xml_str = String::from_utf8_lossy(&file_data);
        let is_calls = xml_str.contains("<calls") || xml_str.contains("<call ");
        let is_mms = xml_str.contains("<mms") || xml_str.contains("<mmses");

        let result = if is_calls {
            // Parse calls
            match parse_calls_xml(Cursor::new(&file_data)) {
                Ok(calls) => {
                    let total = calls.len();
                    match insert_calls(&state.pool, calls, &user_id, &file_name).await {
                        Ok((imp, skip)) => Ok(("calls", total, imp, skip)),
                        Err(e) => Err(format!("Import failed: {}", e)),
                    }
                }
                Err(e) => Err(format!("Failed to parse calls XML: {}", e)),
            }
        } else if is_mms {
            // Parse MMS
            match parse_mms_xml(Cursor::new(&file_data)) {
                Ok(messages) => {
                    let total = messages.len();
                    match insert_mms(&state.pool, messages, &user_id, &file_name).await {
                        Ok((imp, skip)) => Ok(("mms", total, imp, skip)),
                        Err(e) => Err(format!("Import failed: {}", e)),
                    }
                }
                Err(e) => Err(format!("Failed to parse MMS XML: {}", e)),
            }
        } else {
            // Parse SMS
            match parse_sms_xml(Cursor::new(&file_data)) {
                Ok(messages) => {
                    let total = messages.len();
                    match insert_sms(&state.pool, messages, &user_id, &file_name).await {
                        Ok((imp, skip)) => Ok(("sms", total, imp, skip)),
                        Err(e) => Err(format!("Import failed: {}", e)),
                    }
                }
                Err(e) => Err(format!("Failed to parse SMS XML: {}", e)),
            }
        };

        match result {
            Ok((file_type, total, imp, skip)) => {
                total_records += total;
                total_imported += imp;
                total_skipped += skip;
                files_imported += 1;
                results.push(DirectoryImportFileResult {
                    file_name,
                    file_type: file_type.to_string(),
                    total_records: total,
                    imported: imp,
                    skipped: skip,
                    status: "done".to_string(),
                    error: None,
                });
            }
            Err(err_msg) => {
                files_failed += 1;
                results.push(DirectoryImportFileResult {
                    file_name,
                    file_type: "unknown".to_string(),
                    total_records: 0,
                    imported: 0,
                    skipped: 0,
                    status: "error".to_string(),
                    error: Some(err_msg),
                });
            }
        }
    }

    // Auto-link all imported SMS/call data to contacts
    crate::contact_reconciliation::auto_link_after_sms_import(&state.pool, user.id).await;

    let elapsed_seconds = start.elapsed().as_secs_f64();

    Ok(Json(DirectoryImportResponse {
        path: body.path,
        total_files: xml_files.len(),
        files_imported,
        files_failed,
        total_records,
        total_imported,
        total_skipped,
        elapsed_seconds,
        results,
    }))
}
