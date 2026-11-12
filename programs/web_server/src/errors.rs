//! Error mapping Redis error code -> HTTP status (doc/programs/web_server.md#error-mapping).
//! The response body is always the error object { code, message, details }.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use targetscope_common::bus::BusError;

#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    /// Redis error code, or a web-only code: auth_required, wrong_password, too_many_attempts, too_large
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

impl From<BusError> for ApiError {
    fn from(e: BusError) -> Self {
        let _ = e;
        todo!()
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        todo!()
    }
}

/// invalid_message, validation_failed → 400; not_found → 404; in_use, invalid_state,
/// setting_not_set → 409; unavailable → 503; processing_failed → 422; timeout → 504;
/// io_error, database_error → 500.
pub fn status_for(code: &str) -> StatusCode {
    let _ = code;
    todo!()
}
