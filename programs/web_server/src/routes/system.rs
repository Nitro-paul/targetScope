//! GET /api/system (launcher_status) · POST /api/system/programs/{name}/restart (admin)

use axum::Router;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    todo!()
}
