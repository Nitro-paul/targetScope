//! GET /api/settings (admin_password_hash filtered out) · PUT /api/settings/{name} (admin) ·
//! POST /api/settings/reset (admin) · PUT /api/settings/image_dir (change_image_dir, admin).
//! PUT /api/settings/admin_password_hash is refused (400).

use axum::Router;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    todo!()
}
