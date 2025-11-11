mod acl;
mod android_import;
mod android_import_routes;
mod api;
mod attachment_routes;
mod auth;
mod auth_routes;
mod calendar_routes;
mod concept_routes;
mod dashboard_routes;
mod group_routes;
mod import_routes;
mod observability;
mod rate_limit;
mod search_history_routes;
mod security_headers;
mod share_routes;
mod twilio_routes;
mod update_system;
mod state;
mod update_routes;
mod validation;
mod websocket;
mod worker_routes;
mod ws;

use ai_middleware::SegmindClient;
use axum::{
    extract::DefaultBodyLimit,
    http::StatusCode,
    middleware,
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Router,
};
use local_store::LocalStore;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize structured logging
    observability::init_logging();

    // Initialize Prometheus metrics
    let prometheus_handle = observability::init_metrics();

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:./data/contacts.db".to_string());

    let store = LocalStore::new(&database_url).await?;
    let ai_client = SegmindClient::new(Some("mock-api-key".to_string()));

    // Create shared pool
    let pool = Arc::new(store.pool().clone());

    // Create auth and ACL services
    let auth_service = Arc::new(auth::AuthService::new(pool.clone()));
    let acl_service = Arc::new(acl::AclService::new(pool.clone()));

    // Create WebSocket broadcaster
    let ws_broadcaster = Arc::new(websocket::WebSocketBroadcaster::new());

    // Create import jobs (shared state for import routes)
    let import_jobs = Arc::new(tokio::sync::RwLock::new(Vec::new()));

    // Create update system state
    let update_checker = Arc::new(update_system::UpdateChecker::default());
    let update_config = Arc::new(tokio::sync::RwLock::new(update_system::UpdateConfig::default()));
    let last_update_check = Arc::new(tokio::sync::RwLock::new(None));

    let app_state = state::AppState::new(
        store,
        ai_client,
        auth_service,
        acl_service,
        pool,
        ws_broadcaster,
        import_jobs,
        update_checker,
        update_config,
        last_update_check,
    );

    // Metrics endpoint handler
    let metrics_handle = prometheus_handle.clone();
    let metrics_handler = move || {
        let metrics_handle = metrics_handle.clone();
        async move { metrics_handle.render() }
    };

    // Create rate limiters for different route groups
    let auth_rate_limiter =
        rate_limit::create_rate_limiter(rate_limit::RateLimitConfig::auth_routes());
    let attachment_rate_limiter =
        rate_limit::create_rate_limiter(rate_limit::RateLimitConfig::attachment_routes());
    let search_rate_limiter =
        rate_limit::create_rate_limiter(rate_limit::RateLimitConfig::search_routes());
    let import_rate_limiter =
        rate_limit::create_rate_limiter(rate_limit::RateLimitConfig::attachment_routes()); // Similar to attachments (file uploads)

    // Auth routes with strict rate limiting (10 req/min)
    let auth_routes = Router::new()
        .route("/api/auth/signup", post(auth_routes::signup))
        .route("/api/auth/login", post(auth_routes::login))
        .route("/api/auth/refresh", post(auth_routes::refresh))
        .route("/api/auth/logout", post(auth_routes::logout))
        .route("/api/auth/me", get(auth_routes::get_current_user))
        .layer(middleware::from_fn(move |req, next| {
            let limiter = auth_rate_limiter.clone();
            async move { limiter.middleware(req, next).await }
        }))
        .with_state(app_state.clone());

    // Attachment routes with rate limiting (100 req/hr)
    let attachment_routes = Router::new()
        .route(
            "/api/attachments/upload",
            post(attachment_routes::upload_attachment),
        )
        .route("/api/attachments", get(attachment_routes::list_attachments))
        .route(
            "/api/attachments/:id",
            get(attachment_routes::download_attachment)
                .delete(attachment_routes::delete_attachment),
        )
        .layer(middleware::from_fn(move |req, next| {
            let limiter = attachment_rate_limiter.clone();
            async move { limiter.middleware(req, next).await }
        }))
        .with_state(app_state.clone());

    // Search routes with rate limiting (30 req/min)
    let search_routes = Router::new()
        .route("/api/contacts/search", post(api::search_contacts))
        .route(
            "/api/concepts/search",
            post(concept_routes::search_concepts),
        )
        .layer(middleware::from_fn(move |req, next| {
            let limiter = search_rate_limiter.clone();
            async move { limiter.middleware(req, next).await }
        }))
        .with_state(app_state.clone());

    // Import routes with rate limiting (similar to attachments - file uploads)
    let import_router = import_routes::import_routes()
        .layer(middleware::from_fn(move |req, next| {
            let limiter = import_rate_limiter.clone();
            async move { limiter.middleware(req, next).await }
        }))
        .with_state(app_state.clone());

    // Dashboard routes (no rate limiting needed for simple dashboard)
    let dashboard_router = dashboard_routes::dashboard_routes().with_state(app_state.clone());

    // Update system routes
    let update_router = update_routes::update_routes().with_state(app_state.clone());

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/metrics", get(metrics_handler))
        .route("/api/health/detailed", get(detailed_health_check))
        // Merge rate-limited routes
        .merge(auth_routes)
        .merge(attachment_routes)
        .merge(search_routes)
        .merge(import_router)
        .merge(dashboard_router)
        .merge(update_router)
        // Share routes
        .route("/api/shares", post(share_routes::create_share))
        .route("/api/shares/:id/accept", post(share_routes::accept_share))
        .route("/api/shares/:id/reject", post(share_routes::reject_share))
        .route("/api/shares/:id/revoke", post(share_routes::revoke_share))
        .route("/api/shares/sent", get(share_routes::list_sent_shares))
        .route(
            "/api/shares/received",
            get(share_routes::list_received_shares),
        )
        // Group routes
        .route(
            "/api/groups",
            get(group_routes::list_groups).post(group_routes::create_group),
        )
        .route(
            "/api/groups/:id",
            get(group_routes::get_group)
                .put(group_routes::update_group)
                .delete(group_routes::delete_group),
        )
        .route(
            "/api/groups/:id/members/:contact_id",
            post(group_routes::add_member).delete(group_routes::remove_member),
        )
        .route(
            "/api/groups/:id/members",
            post(group_routes::add_member_json),
        )
        // Concept routes
        .route(
            "/api/concepts",
            get(concept_routes::list_concepts).post(concept_routes::create_concept),
        )
        .route(
            "/api/concepts/:id",
            get(concept_routes::get_concept)
                .put(concept_routes::update_concept)
                .delete(concept_routes::delete_concept),
        )
        // Calendar routes
        .route(
            "/api/calendar/events",
            get(calendar_routes::list_events).post(calendar_routes::create_event),
        )
        .route(
            "/api/calendar/events/:id",
            get(calendar_routes::get_event)
                .put(calendar_routes::update_event)
                .delete(calendar_routes::delete_event),
        )
        .route(
            "/api/calendar/events/contact/:contact_id",
            get(calendar_routes::list_events_by_contact),
        )
        // Search history routes
        .route(
            "/api/search-history",
            get(search_history_routes::list_search_history)
                .delete(search_history_routes::clear_search_history),
        )
        .route(
            "/api/search-history/:id",
            get(search_history_routes::get_search_history)
                .delete(search_history_routes::delete_search_history),
        )
        .route(
            "/api/search-history/:id/clicked",
            post(search_history_routes::update_clicked_result),
        )
        // Worker callback routes
        .route(
            "/api/communication/:id/status",
            post(worker_routes::update_communication_status),
        )
        .route(
            "/api/communication/batch-status",
            post(worker_routes::batch_update_communication_status),
        )
        .route(
            "/api/worker/health",
            post(worker_routes::worker_health_check),
        )
        .route("/api/worker/register", post(worker_routes::register_worker))
        // Twilio webhook routes (no auth - Twilio validates via their own methods)
        .route(
            "/api/webhooks/twilio/sms",
            post(twilio_routes::receive_sms),
        )
        .route(
            "/api/webhooks/twilio/sms/test",
            get(twilio_routes::test_webhook),
        )
        // WebSocket route
        .route("/ws/events", get(websocket::websocket_handler))
        // Android backup import routes
        .route(
            "/api/import/android-calls",
            post(android_import_routes::import_android_calls),
        )
        .route(
            "/api/import/android-sms",
            post(android_import_routes::import_android_sms),
        )
        .route(
            "/api/communications/history/:contact_id",
            get(android_import_routes::get_contact_communications),
        )
        .route(
            "/api/communications/search",
            get(android_import_routes::search_communications),
        )
        // Existing routes
        .route(
            "/api/contacts",
            get(api::list_contacts).post(api::create_contact),
        )
        .route(
            "/api/contacts/:id",
            get(api::get_contact)
                .put(api::update_contact)
                .delete(api::delete_contact),
        )
        .route("/api/tags", get(api::list_tags).post(api::create_tag))
        .route(
            "/api/tags/:id",
            put(api::update_tag).delete(api::delete_tag),
        )
        .route(
            "/api/projects",
            get(api::list_projects).post(api::create_project),
        )
        .route(
            "/api/projects/:id",
            get(api::get_project)
                .put(api::update_project)
                .delete(api::delete_project),
        )
        .route("/api/notes", post(api::create_note))
        .route(
            "/api/notes/:id",
            put(api::update_note).delete(api::delete_note),
        )
        .route("/api/notes/contact/:id", get(api::list_notes_by_contact))
        .route("/api/communication", post(api::queue_communication))
        .route(
            "/api/communications",
            get(api::list_communication_attempts).post(api::queue_communication),
        )
        .route(
            "/api/communications/:id/cancel",
            post(api::cancel_communication),
        )
        .route("/api/share", post(api::create_share))
        .route("/api/ai/suggestions/:contact_id", get(api::get_suggestions))
        // SECURITY FIX: Removed unauthenticated legacy WebSocket endpoint
        // Clients should use the authenticated /ws/events endpoint instead
        .layer(DefaultBodyLimit::max(500 * 1024 * 1024)) // 500MB limit for file uploads
        .layer(middleware::from_fn(
            security_headers::security_headers_middleware,
        ))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3002);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    tracing::info!("SagensContact Sync Service listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}

async fn detailed_health_check(
    axum::extract::State(state): axum::extract::State<state::AppState>,
) -> Response {
    use serde_json::json;

    // Check database connection
    let db_healthy = sqlx::query("SELECT 1")
        .fetch_one(state.pool.as_ref())
        .await
        .is_ok();

    // Get database pool stats
    let pool_stats = state.pool.options();

    // Build health response
    let health_status = json!({
        "status": if db_healthy { "healthy" } else { "unhealthy" },
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION"),
        "checks": {
            "database": {
                "status": if db_healthy { "up" } else { "down" },
                "pool": {
                    "max_connections": pool_stats.get_max_connections(),
                    "min_connections": pool_stats.get_min_connections(),
                }
            },
            "storage": {
                "status": "up",
                "backend": "local"
            }
        }
    });

    let status_code = if db_healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (status_code, axum::Json(health_status)).into_response()
}
