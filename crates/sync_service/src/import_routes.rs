use crate::auth::AuthUser;
use crate::state::{AppState, ImportJob, ImportProgress, ImportResult, JobStatus};
use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use core_domain::{Contact, ShareEntityType};
use import_service::{
    create_default_registry, ConnectorMetadata, DuplicateStrategy, MatchCriteria,
};
use local_store::repositories::ContactRepository;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct PreviewQuery {
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImportRequest {
    pub connector_id: Option<String>,
    pub dedupe_strategy: Option<String>,
    pub match_criteria: Option<String>,
    pub dry_run: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct PreviewResponse {
    pub connector: ConnectorMetadata,
    pub preview_rows: Vec<serde_json::Value>,
    pub suggested_mappings: Vec<(String, String)>,
    pub warnings: Vec<String>,
    pub total_rows: usize,
}

/// GET /api/import/connectors - List all available connectors
pub async fn list_connectors(AuthUser(_user): AuthUser) -> impl IntoResponse {
    let registry = create_default_registry();
    let connectors = registry.list_connectors();
    Json(connectors)
}

/// POST /api/import/preview - Upload file and get preview
pub async fn preview_import(
    AuthUser(_user): AuthUser,
    Query(params): Query<PreviewQuery>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let limit = params.limit.unwrap_or(10);

    // Extract file from multipart
    let mut file_data = Vec::new();
    let mut file_name = String::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
    {
        if let Some(name) = field.file_name() {
            file_name = name.to_string();
        }

        let data = field
            .bytes()
            .await
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
        file_data.extend_from_slice(&data);
    }

    if file_data.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "No file uploaded".to_string()));
    }

    // Write to temp file
    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("import_{}", file_name));
    tokio::fs::write(&temp_path, &file_data)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Find connector
    let registry = create_default_registry();
    let connector = registry.find_connector(&temp_path).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            "No suitable connector found".to_string(),
        )
    })?;

    // Parse and preview
    let preview = connector
        .get_preview(&temp_path, limit)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let parse_result = connector
        .parse(&temp_path)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Convert rows to JSON
    let preview_rows: Vec<serde_json::Value> = preview
        .rows
        .iter()
        .map(|row| serde_json::to_value(row).unwrap_or_default())
        .collect();

    let response = PreviewResponse {
        connector: connector.metadata(),
        preview_rows,
        suggested_mappings: preview.suggested_mappings,
        warnings: preview.warnings,
        total_rows: parse_result.rows.len(),
    };

    // Cleanup temp file
    let _ = tokio::fs::remove_file(&temp_path).await;

    Ok(Json(response))
}

/// POST /api/import/execute - Execute import with options
pub async fn execute_import(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Extract file and config from multipart
    let mut file_data = Vec::new();
    let mut file_name = String::new();
    let mut request = ImportRequest {
        connector_id: None,
        dedupe_strategy: None,
        match_criteria: None,
        dry_run: None,
    };

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
    {
        let field_name = field.name().unwrap_or("").to_string();

        if field.file_name().is_some() {
            file_name = field.file_name().unwrap_or("unknown").to_string();
            let data = field
                .bytes()
                .await
                .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
            file_data.extend_from_slice(&data);
        } else {
            // Handle config fields
            let value = field
                .text()
                .await
                .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
            if field_name.as_str() == "config" {
                // Parse JSON config
                request = serde_json::from_str(&value).map_err(|e| {
                    (
                        StatusCode::BAD_REQUEST,
                        format!("Invalid config JSON: {}", e),
                    )
                })?;
            }
        }
    }

    // Create job
    let job_id = Uuid::new_v4();
    let connector_id = request
        .connector_id
        .clone()
        .unwrap_or_else(|| "auto".to_string());

    let user_id = _user.id;
    let job = ImportJob {
        id: job_id,
        user_id,
        file_name: file_name.clone(),
        connector_id: connector_id.clone(),
        status: JobStatus::Pending,
        progress: ImportProgress {
            current: 0,
            total: 0,
            phase: "Queued".to_string(),
            warnings: vec![],
        },
        result: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // Add to job queue
    {
        let mut jobs = state.import_jobs.write().await;
        jobs.push(job.clone());
    }

    // Spawn background task with user_id for proper ownership
    let state_clone = state.clone();
    let request_clone = request.clone();
    tokio::spawn(async move {
        process_import(
            state_clone,
            job_id,
            file_data,
            file_name,
            request_clone,
            user_id,
        )
        .await;
    });

    Ok(Json(serde_json::json!({
        "job_id": job_id,
        "status": "pending",
        "message": "Import job created successfully"
    })))
}

/// GET /api/import/jobs/:job_id - Get job status
pub async fn get_job_status(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(job_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let jobs = state.import_jobs.read().await;
    let job = jobs
        .iter()
        .find(|j| j.id == job_id && j.user_id == user.id)
        .cloned()
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Job not found".to_string()))?;

    Ok(Json(job))
}

/// GET /api/import/jobs - List import jobs for the authenticated user
pub async fn list_jobs(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let jobs = state.import_jobs.read().await;
    let job_list: Vec<_> = jobs
        .iter()
        .filter(|j| j.user_id == user.id)
        .cloned()
        .collect();
    Ok(Json(job_list))
}

/// POST /api/import/jobs/:job_id/cancel - Cancel an import job owned by the authenticated user
pub async fn cancel_job(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(job_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut jobs = state.import_jobs.write().await;
    if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
        if job.user_id != user.id {
            return Err((StatusCode::NOT_FOUND, "Job not found".to_string()));
        }
        job.status = JobStatus::Cancelled;
        job.updated_at = chrono::Utc::now();
        Ok(Json(serde_json::json!({"status": "cancelled"})))
    } else {
        Err((StatusCode::NOT_FOUND, "Job not found".to_string()))
    }
}

/// Background import processing
async fn process_import(
    state: AppState,
    job_id: Uuid,
    file_data: Vec<u8>,
    file_name: String,
    request: ImportRequest,
    user_id: Uuid,
) {
    let start_time = std::time::Instant::now();

    // Update job status helper
    let update_job = |state: AppState,
                      job_id: Uuid,
                      status: JobStatus,
                      phase: String,
                      current: usize,
                      total: usize| {
        async move {
            let mut jobs = state.import_jobs.write().await;
            if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
                job.status = status;
                job.progress.phase = phase;
                job.progress.current = current;
                job.progress.total = total;
                job.updated_at = chrono::Utc::now();
            }
        }
    };

    // Write temp file
    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("import_{}_{}", job_id, file_name));
    if let Err(e) = tokio::fs::write(&temp_path, &file_data).await {
        tracing::error!("Failed to write temp file: {}", e);
        update_job(
            state.clone(),
            job_id,
            JobStatus::Failed,
            "File write error".to_string(),
            0,
            0,
        )
        .await;
        return;
    }

    // Find connector
    update_job(
        state.clone(),
        job_id,
        JobStatus::Validating,
        "Detecting format".to_string(),
        0,
        0,
    )
    .await;
    let registry = create_default_registry();
    let connector = match registry.find_connector(&temp_path) {
        Some(c) => c,
        None => {
            update_job(
                state.clone(),
                job_id,
                JobStatus::Failed,
                "No suitable connector found".to_string(),
                0,
                0,
            )
            .await;
            let _ = tokio::fs::remove_file(&temp_path).await;
            return;
        }
    };

    // Parse file
    update_job(
        state.clone(),
        job_id,
        JobStatus::Parsing,
        "Parsing file".to_string(),
        0,
        0,
    )
    .await;
    let parse_result = match connector.parse(&temp_path).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Parse error: {}", e);
            update_job(
                state.clone(),
                job_id,
                JobStatus::Failed,
                format!("Parse error: {}", e),
                0,
                0,
            )
            .await;
            let _ = tokio::fs::remove_file(&temp_path).await;
            return;
        }
    };

    let total_rows = parse_result.rows.len();
    update_job(
        state.clone(),
        job_id,
        JobStatus::Deduplicating,
        "Checking duplicates".to_string(),
        0,
        total_rows,
    )
    .await;

    // Check for existing contacts in database (fetch all user's contacts for dedup)
    let contact_repo = ContactRepository::new(state.store.pool());
    let existing_contacts = match contact_repo.list(10000, 0, user_id).await {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Failed to fetch existing contacts: {}", e);
            Vec::new()
        }
    };

    // Build lookup maps for deduplication
    let existing_by_email: HashMap<String, &Contact> = existing_contacts
        .iter()
        .filter_map(|c| c.email.as_ref().map(|e| (e.to_lowercase(), c)))
        .collect();
    let existing_by_phone: HashMap<String, &Contact> = existing_contacts
        .iter()
        .filter_map(|c| c.phone.as_ref().map(|p| (normalize_phone(p), c)))
        .collect();

    // Determine dedupe strategy
    let dedupe_strategy = match request.dedupe_strategy.as_deref() {
        Some("skip") => DuplicateStrategy::Skip,
        Some("update") => DuplicateStrategy::Update,
        Some("merge") => DuplicateStrategy::Merge,
        Some("keep_both") => DuplicateStrategy::KeepBoth,
        _ => DuplicateStrategy::Skip,
    };

    let match_criteria = match request.match_criteria.as_deref() {
        Some("email") => MatchCriteria::Email,
        Some("phone") => MatchCriteria::Phone,
        Some("name") => MatchCriteria::FullName,
        _ => MatchCriteria::EmailOrPhone,
    };

    // Import contacts
    update_job(
        state.clone(),
        job_id,
        JobStatus::Importing,
        "Importing contacts".to_string(),
        0,
        total_rows,
    )
    .await;

    let mut imported = 0;
    let mut skipped = 0;
    let mut failed = 0;
    let mut duplicates_found = 0;

    for (index, row) in parse_result.rows.iter().enumerate() {
        // Update progress every 10 rows
        if index % 10 == 0 {
            update_job(
                state.clone(),
                job_id,
                JobStatus::Importing,
                format!("Importing {} of {}", index, total_rows),
                index,
                total_rows,
            )
            .await;
        }

        // Convert row to contact
        let contact = row_to_contact(row, user_id);

        // Check for duplicates
        let is_duplicate = match match_criteria {
            MatchCriteria::Email => contact
                .email
                .as_ref()
                .is_some_and(|e| existing_by_email.contains_key(&e.to_lowercase())),
            MatchCriteria::Phone => contact
                .phone
                .as_ref()
                .is_some_and(|p| existing_by_phone.contains_key(&normalize_phone(p))),
            MatchCriteria::FullName => existing_contacts.iter().any(|c| {
                c.first_name.to_lowercase() == contact.first_name.to_lowercase()
                    && c.last_name.as_ref().map(|l| l.to_lowercase())
                        == contact.last_name.as_ref().map(|l| l.to_lowercase())
            }),
            MatchCriteria::EmailOrPhone | MatchCriteria::Custom(_) => {
                // Default: match by email or phone
                contact
                    .email
                    .as_ref()
                    .is_some_and(|e| existing_by_email.contains_key(&e.to_lowercase()))
                    || contact
                        .phone
                        .as_ref()
                        .is_some_and(|p| existing_by_phone.contains_key(&normalize_phone(p)))
            }
        };

        if is_duplicate {
            duplicates_found += 1;
            match dedupe_strategy {
                DuplicateStrategy::Skip | DuplicateStrategy::Ask => {
                    // Skip duplicates (Ask behaves like Skip in automated import)
                    skipped += 1;
                    continue;
                }
                DuplicateStrategy::KeepBoth => {
                    // Fall through to create new contact
                }
                DuplicateStrategy::Update | DuplicateStrategy::Merge => {
                    // For now, skip - proper update/merge requires finding the existing contact
                    skipped += 1;
                    continue;
                }
            }
        }

        // Dry run mode - don't actually save
        if request.dry_run.unwrap_or(false) {
            imported += 1;
            continue;
        }

        // Create contact
        match contact_repo.create(&contact).await {
            Ok(_) => {
                // Create ACL for the contact
                let _ = state
                    .acl_service
                    .create_acl(&user_id, ShareEntityType::Contact, &contact.id)
                    .await;
                imported += 1;
            }
            Err(e) => {
                tracing::error!("Failed to create contact: {}", e);
                failed += 1;
            }
        }
    }

    // Complete
    let elapsed = start_time.elapsed().as_secs_f64();
    let mut jobs = state.import_jobs.write().await;
    if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
        job.status = JobStatus::Completed;
        job.progress.current = total_rows;
        job.progress.total = total_rows;
        job.progress.phase = "Completed".to_string();
        job.result = Some(ImportResult {
            imported,
            skipped,
            failed,
            duplicates_found,
            elapsed_seconds: elapsed,
            log_id: Uuid::new_v4(),
        });
        job.updated_at = chrono::Utc::now();
    }

    tracing::info!(
        "Import job {} completed: {} imported, {} skipped, {} failed, {} duplicates",
        job_id,
        imported,
        skipped,
        failed,
        duplicates_found
    );

    // Cleanup
    let _ = tokio::fs::remove_file(&temp_path).await;
}

/// Convert a parsed row (HashMap) to a Contact entity
fn row_to_contact(row: &HashMap<String, String>, user_id: Uuid) -> Contact {
    let now = chrono::Utc::now();

    // Standard field mappings (handles common variations)
    let first_name = get_field(
        row,
        &[
            "first_name",
            "firstName",
            "First Name",
            "given_name",
            "Given Name",
            "name",
        ],
    )
    .unwrap_or_else(|| "Unknown".to_string());
    let last_name = get_field(
        row,
        &[
            "last_name",
            "lastName",
            "Last Name",
            "family_name",
            "Family Name",
            "surname",
        ],
    );
    let email = get_field(
        row,
        &["email", "Email", "E-mail", "email_address", "Email Address"],
    );
    let phone = get_field(
        row,
        &[
            "phone",
            "Phone",
            "phone_number",
            "Phone Number",
            "mobile",
            "Mobile",
            "cell",
        ],
    );
    let organization = get_field(
        row,
        &["organization", "Organization", "company", "Company", "org"],
    );
    let title = get_field(
        row,
        &[
            "title",
            "Title",
            "job_title",
            "Job Title",
            "position",
            "Position",
        ],
    );
    let notes = get_field(
        row,
        &["notes", "Notes", "description", "Description", "bio", "Bio"],
    );

    Contact {
        id: Uuid::new_v4(),
        first_name,
        last_name,
        email,
        phone,
        organization,
        title,
        notes,
        social_handles: Vec::new(),
        tags: Vec::new(),
        projects: Vec::new(),
        groups: Vec::new(),
        created_at: now,
        updated_at: now,
        created_by: user_id,
        version: 1,
        last_synced_at: None,
        metadata: serde_json::json!({"imported": true, "import_time": now}),
    }
}

/// Get a field value from a row, trying multiple possible column names
fn get_field(row: &HashMap<String, String>, keys: &[&str]) -> Option<String> {
    for key in keys {
        // Try exact match
        if let Some(val) = row.get(*key) {
            if !val.is_empty() {
                return Some(val.clone());
            }
        }
        // Try case-insensitive match
        let key_lower = key.to_lowercase();
        for (k, v) in row {
            if k.to_lowercase() == key_lower && !v.is_empty() {
                return Some(v.clone());
            }
        }
    }
    None
}

/// Normalize phone number for comparison (remove non-digits)
fn normalize_phone(phone: &str) -> String {
    phone.chars().filter(|c| c.is_ascii_digit()).collect()
}

/// GET /api/import/history - Get import history from database for the authenticated user
pub async fn get_import_history(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    // Build parameterized query to prevent SQL injection
    // Always scope to the authenticated user
    let mut query_builder = sqlx::QueryBuilder::new("SELECT * FROM import_logs WHERE user_id = ");
    query_builder.push_bind(user.id.to_string());

    // Apply filters with parameterized values
    if let Some(status) = params.get("status") {
        query_builder.push(" AND status = ");
        query_builder.push_bind(status);
    }
    if let Some(connector_id) = params.get("connector_id") {
        query_builder.push(" AND connector_id = ");
        query_builder.push_bind(connector_id);
    }
    if let Some(start_date) = params.get("start_date") {
        query_builder.push(" AND started_at >= ");
        query_builder.push_bind(start_date);
    }
    if let Some(end_date) = params.get("end_date") {
        query_builder.push(" AND started_at <= ");
        query_builder.push_bind(end_date);
    }

    query_builder.push(" ORDER BY started_at DESC LIMIT 100");

    match query_builder.build().fetch_all(state.pool.as_ref()).await {
        Ok(rows) => {
            let history: Vec<serde_json::Value> = rows.iter().map(|row| {
                serde_json::json!({
                    "id": row.try_get::<String, _>("id").unwrap_or_default(),
                    "job_id": row.try_get::<String, _>("job_id").unwrap_or_default(),
                    "file_name": row.try_get::<String, _>("file_name").unwrap_or_default(),
                    "connector_id": row.try_get::<String, _>("connector_id").unwrap_or_default(),
                    "total_rows": row.try_get::<i64, _>("total_rows").unwrap_or(0),
                    "imported": row.try_get::<i64, _>("imported").unwrap_or(0),
                    "skipped": row.try_get::<i64, _>("skipped").unwrap_or(0),
                    "failed": row.try_get::<i64, _>("failed").unwrap_or(0),
                    "status": row.try_get::<String, _>("status").unwrap_or_default(),
                    "started_at": row.try_get::<String, _>("started_at").unwrap_or_default(),
                    "completed_at": row.try_get::<Option<String>, _>("completed_at").unwrap_or(None),
                })
            }).collect();
            (StatusCode::OK, Json(history)).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to fetch import history: {}", e);
            (StatusCode::OK, Json(Vec::<serde_json::Value>::new())).into_response()
        }
    }
}

/// Router configuration
pub fn import_routes() -> axum::Router<AppState> {
    axum::Router::new()
        .route(
            "/api/import/connectors",
            axum::routing::get(list_connectors),
        )
        .route("/api/import/preview", axum::routing::post(preview_import))
        .route("/api/import/execute", axum::routing::post(execute_import))
        .route("/api/import/jobs", axum::routing::get(list_jobs))
        .route(
            "/api/import/jobs/:job_id",
            axum::routing::get(get_job_status),
        )
        .route(
            "/api/import/jobs/:job_id/cancel",
            axum::routing::post(cancel_job),
        )
        .route(
            "/api/import/history",
            axum::routing::get(get_import_history),
        )
}
