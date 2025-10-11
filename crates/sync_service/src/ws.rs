use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade},
    response::Response,
};
use tracing::info;

pub async fn ws_handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    info!("WebSocket connection established");

    while let Some(msg) = socket.recv().await {
        match msg {
            Ok(msg) => {
                info!("Received WebSocket message: {:?}", msg);
            }
            Err(e) => {
                info!("WebSocket error: {}", e);
                break;
            }
        }
    }

    info!("WebSocket connection closed");
}
