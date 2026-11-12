//! GET · POST /api/target-types, PUT · DELETE /api/target-types/{id} (delete: admin) ·
//! POST /api/target-types/photo (multipart; find_paper) · POST /api/target-types/photo/capture
//! (capture_raw_image, then find_paper) · POST /api/target-types/photo/prepare (prepare_target_photo)

use axum::Router;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    todo!()
}
