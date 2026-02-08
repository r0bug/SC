use crate::acl::AclService;
use crate::audit::AuditService;
use crate::auth::AuthService;
use crate::update_system::{UpdateChecker, UpdateConfig, UpdateInfo};
use crate::virus_scanner::VirusScanner;
use crate::websocket::WebSocketBroadcaster;
use ai_middleware::SegmindClient;
use local_store::DbPool;
use local_store::LocalStore;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

// Import job types (used by both state and import_routes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportJob {
    pub id: Uuid,
    pub user_id: Uuid,
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

/// Configuration struct for constructing AppState, reducing argument count.
pub struct AppStateConfig {
    pub store: LocalStore,
    pub ai_client: SegmindClient,
    pub auth_service: Arc<AuthService>,
    pub acl_service: Arc<AclService>,
    pub audit_service: Arc<AuditService>,
    pub virus_scanner: Arc<VirusScanner>,
    pub pool: Arc<DbPool>,
    pub ws_broadcaster: Arc<WebSocketBroadcaster>,
    pub import_jobs: Arc<RwLock<Vec<ImportJob>>>,
    pub user_settings: Arc<RwLock<serde_json::Value>>,
    pub update_checker: Arc<UpdateChecker>,
    pub update_config: Arc<RwLock<UpdateConfig>>,
    pub last_update_check: Arc<RwLock<Option<UpdateInfo>>>,
}

#[derive(Clone)]
pub struct AppState {
    pub store: Arc<LocalStore>,
    pub ai_client: Arc<RwLock<SegmindClient>>,
    pub auth_service: Arc<AuthService>,
    pub acl_service: Arc<AclService>,
    pub audit_service: Arc<AuditService>,
    pub virus_scanner: Arc<VirusScanner>,
    pub pool: Arc<DbPool>,
    pub ws_broadcaster: Arc<WebSocketBroadcaster>,
    pub import_jobs: Arc<RwLock<Vec<ImportJob>>>,
    pub user_settings: Arc<RwLock<serde_json::Value>>,
    // Update system state
    pub update_checker: Arc<UpdateChecker>,
    pub update_config: Arc<RwLock<UpdateConfig>>,
    pub last_update_check: Arc<RwLock<Option<UpdateInfo>>>,
}

impl AppState {
    pub fn new(config: AppStateConfig) -> Self {
        Self {
            store: Arc::new(config.store),
            ai_client: Arc::new(RwLock::new(config.ai_client)),
            auth_service: config.auth_service,
            acl_service: config.acl_service,
            audit_service: config.audit_service,
            virus_scanner: config.virus_scanner,
            pool: config.pool,
            ws_broadcaster: config.ws_broadcaster,
            import_jobs: config.import_jobs,
            user_settings: config.user_settings,
            update_checker: config.update_checker,
            update_config: config.update_config,
            last_update_check: config.last_update_check,
        }
    }
}

// AppState already implements Clone, so FromRef is automatic
