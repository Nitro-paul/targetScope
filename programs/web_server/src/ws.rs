// doc/programs/web_server.md#websocket-events

use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use serde_json::Value;
use targetscope_common::bus::{Bus, ReadMode};
use tokio::sync::broadcast;

use crate::state::AppState;

pub const SOURCES: [&str; 5] = ["session_manager", "image_processor", "camera_reader", "launcher", "database_handler"];
pub const BUFFER: usize = 256;
const RESYNC: &str = r#"{"type":"resync"}"#;

pub fn router() -> Router<AppState> {
    Router::new().route("/ws", get(upgrade))
}

async fn upgrade(State(st): State<AppState>, ws: WebSocketUpgrade) -> Response {
    let rx = st.events.subscribe();
    ws.on_upgrade(move |socket| client(socket, rx))
}

async fn client(mut socket: WebSocket, mut rx: broadcast::Receiver<Arc<str>>) {
    loop {
        tokio::select! {
            msg = rx.recv() => {
                let text: Arc<str> = match msg {
                    Ok(t) => t,
                    Err(broadcast::error::RecvError::Lagged(_)) => Arc::from(RESYNC),
                    Err(broadcast::error::RecvError::Closed) => return,
                };
                if socket.send(Message::Text(text.as_ref().into())).await.is_err() {
                    return;
                }
            }
            incoming = socket.recv() => match incoming {
                Some(Ok(Message::Close(_))) | None | Some(Err(_)) => return,
                Some(Ok(_)) => {}
            },
        }
    }
}

// doc/programs/web_server.md#websocket-events
pub async fn read_events(bus: Bus, source: &'static str, tx: broadcast::Sender<Arc<str>>) {
    let after = bus.newest_event_id(source).await.unwrap_or_else(|_| "$".into());
    let mut reader = bus.events::<Value>(source, ReadMode::After(after));
    loop {
        match reader.next().await {
            Ok(event) => {
                let _ = tx.send(Arc::from(event.data.as_str()));
            }
            Err(e) => tracing::warn!(error = %e, source, "events failed"),
        }
    }
}
