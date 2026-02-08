mod acl;
mod android_import;
mod android_import_routes;
mod api;
mod contact_reconciliation;
mod contact_reconciliation_routes;
mod attachment_routes;
mod audit;
mod auth;
mod auth_routes;
mod calendar_routes;
mod communication_concept_routes;
mod concept_detection;
mod concept_routes;
mod label_matcher_routes;
mod dashboard_routes;
mod email_import;
mod email_routes;
mod imap_account_routes;
mod email_triage;
mod email_triage_routes;
mod group_routes;
mod import_routes;
mod location_routes;
mod manual_domain_routes;
mod match_routes;
mod observability;
mod pick_routes;
mod rate_limit;
mod relationship_routes;
mod search_history_routes;
mod security_headers;
mod settings_routes;
mod share_routes;
mod state;
mod twilio_routes;
mod update_routes;
mod update_system;
mod validation;
mod virus_scanner;
mod websocket;
mod worker_routes;
mod ws;

use ai_middleware::SegmindClient;
use axum::{
    extract::DefaultBodyLimit,
    http::{HeaderValue, Method, StatusCode},
    middleware,
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Router,
};
use local_store::LocalStore;
use secure_vault::load_from_env_and_export;
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

    if let Err(err) = load_from_env_and_export() {
        tracing::error!("Failed to load secure vault: {}", err);
        return Err(err.into());
    }

    let mut database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:./data/contacts.db".to_string());

    // Ensure SQLite URLs have create mode and data directory exists
    if database_url.starts_with("sqlite:") {
        if !database_url.contains("mode=rwc") {
            let separator = if database_url.contains('?') { "&" } else { "?" };
            database_url = format!("{}{}mode=rwc", database_url, separator);
        }
        let db_path = database_url.trim_start_matches("sqlite:");
        let db_path = db_path.split('?').next().unwrap_or(db_path);
        if let Some(parent) = std::path::Path::new(db_path).parent() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let store = LocalStore::new(&database_url).await?;
    let ai_key = std::env::var("SEGMIND_API_KEY").ok().filter(|k| !k.is_empty());
    let ai_client = SegmindClient::new(ai_key);

    // Create shared pool
    let pool = Arc::new(store.pool().clone());

    // Create auth, ACL, and audit services
    let auth_service = Arc::new(auth::AuthService::new(pool.clone()));
    let acl_service = Arc::new(acl::AclService::new(pool.clone()));
    let audit_service = Arc::new(audit::AuditService::new(pool.clone()));

    // Create WebSocket broadcaster
    let ws_broadcaster = Arc::new(websocket::WebSocketBroadcaster::new());

    // Create import jobs (shared state for import routes)
    let import_jobs = Arc::new(tokio::sync::RwLock::new(Vec::new()));

    // Create update system state
    let update_checker = Arc::new(update_system::UpdateChecker::default());
    let update_config = Arc::new(tokio::sync::RwLock::new(
        update_system::UpdateConfig::default(),
    ));
    let last_update_check = Arc::new(tokio::sync::RwLock::new(None));

    // Create virus scanner
    let scanner_enabled = std::env::var("VIRUS_SCANNER_ENABLED")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()
        .unwrap_or(true);
    let scanner_socket = std::env::var("CLAMAV_SOCKET_PATH")
        .unwrap_or_else(|_| "/var/run/clamav/clamd.ctl".to_string());
    let scanner_timeout = std::env::var("VIRUS_SCANNER_TIMEOUT")
        .unwrap_or_else(|_| "30".to_string())
        .parse::<u64>()
        .unwrap_or(30);
    let scanner_strict = std::env::var("VIRUS_SCANNER_STRICT")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()
        .unwrap_or(true);
    let virus_scanner = Arc::new(virus_scanner::VirusScanner::new(
        scanner_enabled,
        scanner_socket,
        scanner_timeout,
        scanner_strict,
    ));

    let default_settings = serde_json::json!({
        "theme": "light",
        "notifications": true,
        "sync_enabled": true
    });
    let user_settings = Arc::new(tokio::sync::RwLock::new(default_settings));

    let app_state = state::AppState::new(
        store,
        ai_client,
        auth_service,
        acl_service,
        audit_service,
        virus_scanner,
        pool,
        ws_broadcaster,
        import_jobs,
        user_settings,
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
        .route(
            "/api/auth/me",
            get(auth_routes::get_current_user).put(auth_routes::update_current_user),
        )
        .route(
            "/api/auth/change-password",
            post(auth_routes::change_password),
        )
        .route("/api/auth/logout-all", post(auth_routes::logout_all))
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

    // New concept-graph routes
    let pick_router = pick_routes::pick_routes().with_state(app_state.clone());
    let location_router = location_routes::location_routes().with_state(app_state.clone());
    let relationship_router =
        relationship_routes::relationship_routes().with_state(app_state.clone());
    let match_router = match_routes::match_routes().with_state(app_state.clone());
    let comm_concept_router =
        communication_concept_routes::communication_concept_routes().with_state(app_state.clone());
    let email_router = email_routes::email_routes().with_state(app_state.clone());
    let email_triage_router = email_triage_routes::email_triage_routes().with_state(app_state.clone());
    let manual_domain_router = manual_domain_routes::manual_domain_routes().with_state(app_state.clone());
    let reconciliation_router = contact_reconciliation_routes::reconciliation_routes().with_state(app_state.clone());
    let imap_account_router = imap_account_routes::imap_account_routes().with_state(app_state.clone());
    let label_matcher_router = label_matcher_routes::label_matcher_routes().with_state(app_state.clone());

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
        // Concept-graph routes (picks, locations, relationships, matches, comm concepts)
        .merge(pick_router)
        .merge(location_router)
        .merge(relationship_router)
        .merge(match_router)
        .merge(comm_concept_router)
        // Email routes
        .merge(email_router)
        // Email triage + domain routes
        .merge(email_triage_router)
        // Manual domain assignment routes
        .merge(manual_domain_router)
        // Contact reconciliation routes
        .merge(reconciliation_router)
        // IMAP account management routes
        .merge(imap_account_router)
        // Label matcher routes
        .merge(label_matcher_router)
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
        .route(
            "/api/concepts/:id/children",
            get(concept_routes::list_children),
        )
        .route(
            "/api/concepts/:id/keywords",
            post(concept_routes::add_keyword),
        )
        .route(
            "/api/concepts/:id/keywords/:keyword_id",
            axum::routing::delete(concept_routes::remove_keyword),
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
        .route(
            "/api/settings",
            get(settings_routes::get_settings).put(settings_routes::update_settings),
        )
        .route(
            "/api/settings/ai",
            get(settings_routes::get_ai_status).put(settings_routes::update_ai_key),
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
        .route("/api/webhooks/twilio/sms", post(twilio_routes::receive_sms))
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
            "/api/import/android-sms/directory",
            post(android_import_routes::import_directory),
        )
        .route(
            "/api/communications/history/:contact_id",
            get(android_import_routes::get_contact_communications),
        )
        .route(
            "/api/communications/search",
            get(android_import_routes::search_communications),
        )
        // SMS History routes
        .route(
            "/api/sms-history/conversations",
            get(android_import_routes::list_sms_conversations),
        )
        .route(
            "/api/sms-history/conversation/:phone",
            get(android_import_routes::get_sms_conversation),
        )
        .route(
            "/api/sms-history/stats",
            get(android_import_routes::get_sms_stats),
        )
        .route(
            "/api/sms-history/source/:filename",
            axum::routing::delete(android_import_routes::delete_sms_by_source),
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
        .layer({
            // Configure CORS with specific allowed origins (not permissive)
            let allowed_origins = std::env::var("ALLOWED_ORIGINS").unwrap_or_else(|_| {
                "http://localhost:3001,http://localhost:5173,http://localhost:3000".to_string()
            });

            let origins: Vec<HeaderValue> = allowed_origins
                .split(',')
                .filter_map(|s| s.trim().parse().ok())
                .collect();

            CorsLayer::new()
                .allow_origin(origins)
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::DELETE,
                    Method::OPTIONS,
                ])
                .allow_headers([
                    axum::http::header::AUTHORIZATION,
                    axum::http::header::CONTENT_TYPE,
                ])
                .allow_credentials(true)
        })
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
