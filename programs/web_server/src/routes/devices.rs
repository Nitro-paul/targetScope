//! GET /api/devices · POST /api/devices · PUT /api/devices/{id} · DELETE /api/devices/{id} (admin) ·
//! PUT /api/devices/{id}/image (multipart; set_device_image) · DELETE /api/devices/{id}/image (admin)

use axum::Router;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    todo!()
}
