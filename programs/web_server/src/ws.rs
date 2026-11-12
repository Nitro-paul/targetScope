//! /ws (doc/programs/web_server.md#websocket-events): every event of session_manager,
//! image_processor, camera_reader, launcher and database_handler, unchanged, as text messages.
//! One background task per event stream (plain read) feeds a broadcast channel.

use axum::Router;

use crate::state::AppState;

pub const SOURCES: [&str; 5] = [
    "session_manager",
    "image_processor",
    "camera_reader",
    "launcher",
    "database_handler",
];

pub fn router() -> Router<AppState> {
    todo!()
}

pub async fn read_events(state: AppState) {
    let _ = state;
    todo!()
}
