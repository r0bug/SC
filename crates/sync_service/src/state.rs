use local_store::LocalStore;
use ai_middleware::SegmindClient;
use crate::auth::AuthService;
use crate::acl::AclService;
use crate::websocket::WebSocketBroadcaster;
use sqlx::{Pool, Sqlite};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub store: Arc<LocalStore>,
    pub ai_client: Arc<SegmindClient>,
    pub auth_service: Arc<AuthService>,
    pub acl_service: Arc<AclService>,
    pub pool: Arc<Pool<Sqlite>>,
    pub ws_broadcaster: Arc<WebSocketBroadcaster>,
}

impl AppState {
    pub fn new(
        store: LocalStore,
        ai_client: SegmindClient,
        auth_service: Arc<AuthService>,
        acl_service: Arc<AclService>,
        pool: Arc<Pool<Sqlite>>,
        ws_broadcaster: Arc<WebSocketBroadcaster>,
    ) -> Self {
        Self {
            store: Arc::new(store),
            ai_client: Arc::new(ai_client),
            auth_service,
            acl_service,
            pool,
            ws_broadcaster,
        }
    }
}

// AppState already implements Clone, so FromRef is automatic