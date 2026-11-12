//! REST API (doc/programs/web_server.md#rest-api). All bodies are JSON.

pub mod admin;
pub mod analyze;
pub mod camera;
pub mod devices;
pub mod exports;
pub mod imports;
pub mod saved_sessions;
pub mod session;
pub mod settings;
pub mod shooters;
pub mod statistics;
pub mod system;
pub mod target_types;

use axum::Router;

use crate::state::AppState;

pub fn api_router() -> Router<AppState> {
    todo!()
}
