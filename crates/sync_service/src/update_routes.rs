use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::auth::AuthUser;
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct VersionResponse {
    pub version: String,
    pub build_date: String,
    pub commit_hash: Option<String>,
}

/// GET /api/system/version - Get current version
pub async fn get_version() -> impl IntoResponse {
    let version = env!("CARGO_PKG_VERSION");
    let build_date = env!("BUILD_DATE", "unknown");
    let commit_hash = option_env!("GIT_HASH").map(|s| s.to_string());

    Json(VersionResponse {
        version: version.to_string(),
        build_date: build_date.to_string(),
        commit_hash,
    })
}

/// GET /api/system/updates/check - Check for available updates
pub async fn check_updates(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match state.update_checker.check_for_updates().await {
        Ok(info) => {
            // Cache the result
            let mut last_check = state.last_update_check.write().await;
            *last_check = Some(info.clone());

            Ok(Json(info))
        }
        Err(e) => {
            tracing::error!("Failed to check for updates: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to check for updates: {}", e),
            ))
        }
    }
}

/// GET /api/system/updates/info - Get cached update info
pub async fn get_update_info(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let last_check = state.last_update_check.read().await;

    match last_check.as_ref() {
        Some(info) => Ok(Json(info.clone())),
        None => Err((
            StatusCode::NOT_FOUND,
            "No update check performed yet".to_string(),
        )),
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateConfigRequest {
    pub auto_check: Option<bool>,
    pub auto_download: Option<bool>,
    pub auto_install: Option<bool>,
    pub check_interval_hours: Option<u32>,
}

/// GET /api/system/updates/config - Get update configuration
pub async fn get_update_config(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let config = state.update_config.read().await;
    Json(config.clone())
}

/// PUT /api/system/updates/config - Update configuration
pub async fn update_config(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Json(request): Json<UpdateConfigRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut config = state.update_config.write().await;

    if let Some(auto_check) = request.auto_check {
        config.auto_check = auto_check;
    }
    if let Some(auto_download) = request.auto_download {
        config.auto_download = auto_download;
    }
    if let Some(auto_install) = request.auto_install {
        config.auto_install = auto_install;
    }
    if let Some(interval) = request.check_interval_hours {
        if interval == 0 || interval > 168 {
            return Err((
                StatusCode::BAD_REQUEST,
                "Interval must be between 1 and 168 hours".to_string(),
            ));
        }
        config.check_interval_hours = interval;
    }

    Ok(Json(config.clone()))
}

#[derive(Debug, Serialize)]
pub struct UpdateStatus {
    pub status: String,
    pub message: String,
    pub download_progress: Option<f32>,
}

/// POST /api/system/updates/download - Download available update
pub async fn download_update(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {

    let last_check = state.last_update_check.read().await;

    let info = last_check.as_ref().ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            "No update check performed".to_string(),
        )
    })?;

    if !info.update_available {
        return Err((StatusCode::BAD_REQUEST, "No update available".to_string()));
    }

    let download_url = info.download_url.as_ref().ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            "No download URL available".to_string(),
        )
    })?;

    // Create temporary download path
    let temp_dir = std::env::temp_dir();
    let download_path = temp_dir.join(format!("sagenscontact-update-{}", info.latest_version));

    // Download in background
    let checker = state.update_checker.clone();
    let url = download_url.clone();
    let path = download_path.clone();

    tokio::spawn(async move {
        if let Err(e) = checker.download_update(&url, &path).await {
            tracing::error!("Failed to download update: {}", e);
        } else {
            tracing::info!("Successfully downloaded update to {:?}", path);
        }
    });

    Ok(Json(UpdateStatus {
        status: "downloading".to_string(),
        message: format!("Downloading update {}", info.latest_version),
        download_progress: Some(0.0),
    }))
}

/// POST /api/system/updates/apply - Apply downloaded update (requires restart)
pub async fn apply_update(
    AuthUser(_user): AuthUser,
    State(_state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {

    // This endpoint would typically trigger a graceful shutdown and restart
    // For safety, this should require elevated permissions or confirmation

    Ok(Json(UpdateStatus {
        status: "pending_restart".to_string(),
        message: "Update will be applied on next restart".to_string(),
        download_progress: None,
    }))
}

/// Router configuration
pub fn update_routes() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/api/system/version", axum::routing::get(get_version))
        .route(
            "/api/system/updates/check",
            axum::routing::get(check_updates),
        )
        .route(
            "/api/system/updates/info",
            axum::routing::get(get_update_info),
        )
        .route(
            "/api/system/updates/config",
            axum::routing::get(get_update_config).put(update_config),
        )
        .route(
            "/api/system/updates/download",
            axum::routing::post(download_update),
        )
        .route(
            "/api/system/updates/apply",
            axum::routing::post(apply_update),
        )
}
