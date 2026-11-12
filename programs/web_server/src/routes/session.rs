//! Current session → session_manager.
//! GET /api/session (session_get) · POST /api/session/setup · PATCH /api/session/setup ·
//! PUT /api/session/notes · POST /api/session/new-paper · POST /api/session/calibrate ·
//! POST /api/session/{start,pause,resume,stop,save,cancel} · POST /api/session/shots ·
//! PUT|DELETE /api/session/shots/{index} · POST /api/session/photo (multipart or JSON)

use axum::Router;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    todo!()
}
