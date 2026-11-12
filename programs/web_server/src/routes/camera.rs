//! GET /api/camera (camera_status) · GET /api/camera/devices · GET /api/camera/controls ·
//! PUT /api/camera/controls/{id} · GET /api/camera/lens · POST /api/camera/lens/{start,capture,finish,cancel} ·
//! DELETE /api/camera/lens · GET /api/calibration (live_calibration_status)

use axum::Router;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    todo!()
}
