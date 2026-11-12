//! Saved sessions (records: read and delete only) → database_handler.
//! POST /api/sessions/query (read_sessions, include_shots = false) · GET /api/sessions/{id} ·
//! DELETE /api/sessions/{id} (admin)

use axum::Router;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    todo!()
}
