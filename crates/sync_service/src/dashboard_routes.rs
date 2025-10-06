use axum::{
    extract::State,
    response::IntoResponse,
    http::StatusCode,
    Json,
};
use serde_json::json;
use std::sync::Arc;
use sqlx::{Pool, Sqlite};

#[derive(Clone)]
pub struct DashboardState {
    pub pool: Arc<Pool<Sqlite>>,
}

/// GET /api/dashboard - Get dashboard summary
pub async fn get_dashboard(
    State(state): State<DashboardState>,
) -> impl IntoResponse {
    // Get counts from database
    let contacts_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM contacts")
        .fetch_one(state.pool.as_ref())
        .await
        .unwrap_or(0);

    let groups_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM groups")
        .fetch_one(state.pool.as_ref())
        .await
        .unwrap_or(0);

    let projects_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM projects")
        .fetch_one(state.pool.as_ref())
        .await
        .unwrap_or(0);

    let notes_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM notes")
        .fetch_one(state.pool.as_ref())
        .await
        .unwrap_or(0);

    let recent_imports = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM import_logs WHERE started_at > datetime('now', '-7 days')"
    )
    .fetch_one(state.pool.as_ref())
    .await
    .unwrap_or(0);

    let dashboard = json!({
        "summary": {
            "contacts": contacts_count,
            "groups": groups_count,
            "projects": projects_count,
            "notes": notes_count,
            "recent_imports": recent_imports,
        },
        "recent_activity": [],
        "quick_stats": {
            "total_contacts": contacts_count,
            "active_projects": projects_count,
        }
    });

    (StatusCode::OK, Json(dashboard)).into_response()
}

pub fn dashboard_routes() -> axum::Router<DashboardState> {
    axum::Router::new()
        .route("/api/dashboard", axum::routing::get(get_dashboard))
}
