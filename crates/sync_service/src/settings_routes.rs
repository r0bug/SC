use crate::auth::AuthUser;
use crate::state::AppState;
use axum::{extract::State, Json};
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
