use crate::auth::AuthUser;
use crate::state::AppState;
use ai_middleware::SegmindClient;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SettingsPayload {
    pub theme: Option<String>,
    pub notifications: Option<bool>,
    #[serde(default)]
    pub sync_enabled: Option<bool>,
}

pub async fn get_settings(
    State(state): State<AppState>,
    AuthUser(_user): AuthUser,
) -> Json<serde_json::Value> {
    let settings = state.user_settings.read().await.clone();
    Json(settings)
}

pub async fn update_settings(
    State(state): State<AppState>,
    AuthUser(_user): AuthUser,
    Json(payload): Json<SettingsPayload>,
) -> Json<serde_json::Value> {
    let mut settings = state.user_settings.write().await;

    if let Some(theme) = payload.theme {
        settings["theme"] = serde_json::json!(theme);
    }

    if let Some(notifications) = payload.notifications {
        settings["notifications"] = serde_json::json!(notifications);
    }

    if let Some(sync_enabled) = payload.sync_enabled {
        settings["sync_enabled"] = serde_json::json!(sync_enabled);
    }

    Json(settings.clone())
}

// --- AI API Key management ---

#[derive(Debug, Deserialize)]
pub struct AiKeyPayload {
    pub api_key: String,
}

/// GET /api/settings/ai - returns whether an AI key is configured (not the key itself)
pub async fn get_ai_status(
    State(state): State<AppState>,
    AuthUser(_user): AuthUser,
) -> Json<serde_json::Value> {
    let client = state.ai_client.read().await;
    let has_key = !client.is_mock_mode();
    Json(serde_json::json!({
        "configured": has_key,
        "provider": "segmind",
        "model": "llama-3.1-8b-instruct"
    }))
}

/// PUT /api/settings/ai - update the AI API key at runtime
pub async fn update_ai_key(
    State(state): State<AppState>,
    AuthUser(_user): AuthUser,
    Json(payload): Json<AiKeyPayload>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let key = payload.api_key.trim().to_string();
    let key_opt = if key.is_empty() { None } else { Some(key) };

    let new_client = SegmindClient::new(key_opt);
    let is_mock = new_client.is_mock_mode();

    // Replace the client
    let mut client = state.ai_client.write().await;
    *client = new_client;

    Ok(Json(serde_json::json!({
        "configured": !is_mock,
        "message": if is_mock { "AI key cleared - running in mock mode" } else { "AI key updated successfully" }
    })))
}
