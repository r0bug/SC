use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use import_service::{
    create_default_registry, ConnectorMetadata, DeduplicationConfig, DeduplicationEngine,
    DuplicateStrategy, MatchCriteria, ParseResult,
};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Clone)]
pub struct ImportState {
    pub pool: Pool<Sqlite>,
    pub jobs: Arc<RwLock<Vec<ImportJob>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportJob {
    pub id: Uuid,
    pub file_name: String,
    pub connector_id: String,
    pub status: JobStatus,
    pub progress: ImportProgress,
    pub result: Option<ImportResult>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Validating,
    Parsing,
    Deduplicating,
    Importing,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportProgress {
    pub current: usize,
    pub total: usize,
    pub phase: String,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub imported: usize,
    pub skipped: usize,
    pub failed: usize,
    pub duplicates_found: usize,
    pub elapsed_seconds: f64,
    pub log_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct PreviewQuery {
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
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
pub async fn list_connectors() -> impl IntoResponse {
    let registry = create_default_registry();
    let connectors = registry.list_connectors();
    Json(connectors)
}

/// POST /api/import/preview - Upload file and get preview
pub async fn preview_import(
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
    std::fs::write(&temp_path, &file_data)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Find connector
    let registry = create_default_registry();
    let connector = registry
        .find_connector(&temp_path)
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "No suitable connector found".to_string()))?;

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
    let _ = std::fs::remove_file(&temp_path);

    Ok(Json(response))
}

/// POST /api/import/execute - Execute import with options
pub async fn execute_import(
    State(state): State<ImportState>,
    Json(request): Json<ImportRequest>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Extract file
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

    // Create job
    let job_id = Uuid::new_v4();
    let connector_id = request.connector_id.unwrap_or_else(|| "auto".to_string());

    let job = ImportJob {
        id: job_id,
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
        let mut jobs = state.jobs.write().await;
        jobs.push(job.clone());
    }

    // Spawn background task
    let state_clone = state.clone();
    let request_clone = request.clone();
    tokio::spawn(async move {
        process_import(state_clone, job_id, file_data, file_name, request_clone).await;
    });

    Ok(Json(serde_json::json!({
        "job_id": job_id,
        "status": "pending",
        "message": "Import job created successfully"
    })))
}

/// GET /api/import/jobs/:job_id - Get job status
pub async fn get_job_status(
    State(state): State<ImportState>,
    Path(job_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let jobs = state.jobs.read().await;
    let job = jobs
        .iter()
        .find(|j| j.id == job_id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Job not found".to_string()))?;

    Ok(Json(job))
}

/// GET /api/import/jobs - List all import jobs
pub async fn list_jobs(
    State(state): State<ImportState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let jobs = state.jobs.read().await;
    let job_list: Vec<_> = jobs.iter().cloned().collect();
    Ok(Json(job_list))
}

/// POST /api/import/jobs/:job_id/cancel - Cancel an import job
pub async fn cancel_job(
    State(state): State<ImportState>,
    Path(job_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut jobs = state.jobs.write().await;
    if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
        job.status = JobStatus::Cancelled;
        job.updated_at = chrono::Utc::now();
        Ok(Json(serde_json::json!({"status": "cancelled"})))
    } else {
        Err((StatusCode::NOT_FOUND, "Job not found".to_string()))
    }
}

/// Background import processing
async fn process_import(
    state: ImportState,
    job_id: Uuid,
    file_data: Vec<u8>,
    file_name: String,
    request: ImportRequest,
) {
    let start_time = std::time::Instant::now();

    // Update job status
    let update_status = |status: JobStatus, phase: &str| async {
        let mut jobs = state.jobs.write().await;
        if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
            job.status = status.clone();
            job.progress.phase = phase.to_string();
            job.updated_at = chrono::Utc::now();
        }
    };

    // Write temp file
    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("import_{}_{}", job_id, file_name));
    if let Err(e) = std::fs::write(&temp_path, &file_data) {
        tracing::error!("Failed to write temp file: {}", e);
        update_status(JobStatus::Failed, "File write error").await;
        return;
    }

    // Find connector
    update_status(JobStatus::Validating, "Detecting format").await;
    let registry = create_default_registry();
    let connector = match registry.find_connector(&temp_path) {
        Some(c) => c,
        None => {
            update_status(JobStatus::Failed, "No suitable connector found").await;
            let _ = std::fs::remove_file(&temp_path);
            return;
        }
    };

    // Parse file
    update_status(JobStatus::Parsing, "Parsing file").await;
    let parse_result = match connector.parse(&temp_path).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Parse error: {}", e);
            update_status(JobStatus::Failed, &format!("Parse error: {}", e)).await;
            let _ = std::fs::remove_file(&temp_path);
            return;
        }
    };

    // Deduplicate
    update_status(JobStatus::Deduplicating, "Checking duplicates").await;
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

    let dedup_config = DeduplicationConfig {
        strategy: dedupe_strategy,
        match_criteria,
        ..Default::default()
    };

    let dedup_engine = DeduplicationEngine::new(dedup_config);
    let duplicates = match dedup_engine.find_duplicates(&parse_result.rows) {
        Ok(d) => d.len(),
        Err(e) => {
            tracing::error!("Deduplication error: {}", e);
            0
        }
    };

    // Import (placeholder - would call actual import logic)
    update_status(JobStatus::Importing, "Importing contacts").await;

    // TODO: Actual import implementation
    let imported = parse_result.rows.len();
    let skipped = duplicates;
    let failed = 0;

    // Complete
    let elapsed = start_time.elapsed().as_secs_f64();
    let mut jobs = state.jobs.write().await;
    if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
        job.status = JobStatus::Completed;
        job.progress.current = imported;
        job.progress.total = imported;
        job.result = Some(ImportResult {
            imported,
            skipped,
            failed,
            duplicates_found: duplicates,
            elapsed_seconds: elapsed,
            log_id: Uuid::new_v4(),
        });
        job.updated_at = chrono::Utc::now();
    }

    // Cleanup
    let _ = std::fs::remove_file(&temp_path);
}

/// Router configuration
pub fn import_routes() -> axum::Router<ImportState> {
    axum::Router::new()
        .route("/connectors", axum::routing::get(list_connectors))
        .route("/preview", axum::routing::post(preview_import))
        .route("/execute", axum::routing::post(execute_import))
        .route("/jobs", axum::routing::get(list_jobs))
        .route("/jobs/:job_id", axum::routing::get(get_job_status))
        .route("/jobs/:job_id/cancel", axum::routing::post(cancel_job))
}
