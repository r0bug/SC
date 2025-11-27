use crate::{auth::AuthUser, state::AppState};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BroadcastEvent {
    ContactCreated {
        id: Uuid,
        user_id: Uuid,
    },
    ContactUpdated {
        id: Uuid,
        user_id: Uuid,
    },
    ContactDeleted {
        id: Uuid,
        user_id: Uuid,
    },

    NoteCreated {
        id: Uuid,
        contact_id: Uuid,
        user_id: Uuid,
    },
    NoteUpdated {
        id: Uuid,
        user_id: Uuid,
    },
    NoteDeleted {
        id: Uuid,
        user_id: Uuid,
    },

    CalendarEventCreated {
        id: Uuid,
        user_id: Uuid,
    },
    CalendarEventUpdated {
        id: Uuid,
        user_id: Uuid,
    },
    CalendarEventDeleted {
        id: Uuid,
        user_id: Uuid,
    },

    GroupCreated {
        id: Uuid,
        user_id: Uuid,
    },
    GroupUpdated {
        id: Uuid,
        user_id: Uuid,
    },
    GroupDeleted {
        id: Uuid,
        user_id: Uuid,
    },

    ConceptCreated {
        id: Uuid,
        user_id: Uuid,
    },
    ConceptUpdated {
        id: Uuid,
        user_id: Uuid,
    },
    ConceptDeleted {
        id: Uuid,
        user_id: Uuid,
    },

    ShareInviteReceived {
        invite_id: Uuid,
        from_user_id: Uuid,
        to_user_id: Uuid,
    },
    ShareAccepted {
        share_id: Uuid,
        user_id: Uuid,
    },
    ShareRejected {
        share_id: Uuid,
        user_id: Uuid,
    },

    /// SMS received via Twilio webhook
    SmsReceived {
        id: Uuid,
        contact_id: Option<Uuid>,
        from: String,
        body: String,
    },
    /// SMS sent to a contact
    SmsSent {
        id: Uuid,
        contact_id: Uuid,
        to: String,
        body: String,
    },
}

type Clients = Arc<RwLock<HashMap<Uuid, mpsc::UnboundedSender<Message>>>>;

#[derive(Clone)]
pub struct WebSocketBroadcaster {
    clients: Clients,
}

impl Default for WebSocketBroadcaster {
    fn default() -> Self {
        Self::new()
    }
}

impl WebSocketBroadcaster {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn broadcast(&self, event: BroadcastEvent) {
        let clients = self.clients.read().await;
        let message = serde_json::to_string(&event).unwrap();

        for (_id, tx) in clients.iter() {
            let _ = tx.send(Message::Text(message.clone()));
        }
    }

    async fn register_client(&self, user_id: Uuid, tx: mpsc::UnboundedSender<Message>) {
        let mut clients = self.clients.write().await;
        clients.insert(user_id, tx);
    }

    async fn unregister_client(&self, user_id: Uuid) {
        let mut clients = self.clients.write().await;
        clients.remove(&user_id);
    }
}

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, app_state, user.id))
}

async fn handle_socket(socket: WebSocket, app_state: AppState, user_id: Uuid) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel();

    app_state.ws_broadcaster.register_client(user_id, tx).await;

    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                if text == "ping" {
                    continue;
                }
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }

    app_state.ws_broadcaster.unregister_client(user_id).await;
}
