use crate::acl::AclService;
use crate::auth::AuthService;
use crate::websocket::WebSocketBroadcaster;
use ai_middleware::SegmindClient;
use local_store::LocalStore;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

// Import job types (used by both state and import_routes)
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

#[derive(Clone)]
pub struct AppState {
    pub store: Arc<LocalStore>,
    pub ai_client: Arc<SegmindClient>,
    pub auth_service: Arc<AuthService>,
    pub acl_service: Arc<AclService>,
    pub pool: Arc<Pool<Sqlite>>,
    pub ws_broadcaster: Arc<WebSocketBroadcaster>,
    pub import_jobs: Arc<RwLock<Vec<ImportJob>>>,
}

impl AppState {
    pub fn new(
        store: LocalStore,
        ai_client: SegmindClient,
        auth_service: Arc<AuthService>,
        acl_service: Arc<AclService>,
        pool: Arc<Pool<Sqlite>>,
        ws_broadcaster: Arc<WebSocketBroadcaster>,
        import_jobs: Arc<RwLock<Vec<ImportJob>>>,
    ) -> Self {
        Self {
            store: Arc::new(store),
            ai_client: Arc::new(ai_client),
            auth_service,
            acl_service,
            pool,
            ws_broadcaster,
            import_jobs,
        }
    }
}

// AppState already implements Clone, so FromRef is automatic
