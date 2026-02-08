use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

use crate::auth::AuthUser;
use crate::contact_reconciliation;
use crate::state::AppState;

/// POST /api/contacts/reconcile - Trigger full contact reconciliation
pub async fn reconcile_contacts(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let result = contact_reconciliation::reconcile_all(&state.pool, user.id).await;

    Ok(Json(serde_json::json!({
        "contacts_created": result.contacts_created,
        "contacts_matched": result.contacts_matched,
        "email_messages_linked": result.email_messages_linked,
        "sms_messages_linked": result.sms_messages_linked,
        "errors": result.errors,
    })))
}

/// GET /api/contacts/unlinked-stats - Get counts of unlinked communications
pub async fn unlinked_stats(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let stats = contact_reconciliation::get_unlinked_stats(&state.pool).await;

    Ok(Json(serde_json::json!({
        "unlinked_email_senders": stats.unlinked_email_senders,
        "unlinked_sms_senders": stats.unlinked_sms_senders,
        "total_unlinked_senders": stats.total_unlinked_senders,
    })))
}

/// Build reconciliation routes as a Router
pub fn reconciliation_routes() -> Router<AppState> {
    Router::new()
        .route("/api/contacts/reconcile", post(reconcile_contacts))
        .route("/api/contacts/unlinked-stats", get(unlinked_stats))
}
