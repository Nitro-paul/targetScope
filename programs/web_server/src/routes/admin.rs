//! GET /api/admin · POST /api/admin/login · POST /api/admin/logout · PUT /api/admin/password

use axum::Router;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    todo!()
}
