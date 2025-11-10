use crate::acl::AclService;
use crate::auth::AuthService;
use crate::import_routes::ImportJob;
use crate::websocket::WebSocketBroadcaster;
use ai_middleware::SegmindClient;
use local_store::LocalStore;
use sqlx::{Pool, Sqlite};
use std::sync::Arc;
use tokio::sync::RwLock;

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
