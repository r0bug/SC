use crate::auth::AuthUser;
use crate::state::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

/// GET /api/dashboard - Get dashboard summary
pub async fn get_dashboard(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // Get counts from database
    let pool = state.store.pool();

    let contacts_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM contacts")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    let groups_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM groups")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    let projects_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM projects")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    let notes_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM notes")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    let recent_imports = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM import_logs WHERE started_at > datetime('now', '-7 days')",
    )
    .fetch_one(pool)
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

pub fn dashboard_routes() -> axum::Router<AppState> {
    axum::Router::new().route("/api/dashboard", axum::routing::get(get_dashboard))
}
