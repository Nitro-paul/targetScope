//! Optional `Idempotency-Key` header (UUID) on write endpoints; used as request_id so the
//! browser can safely resend.

use uuid::Uuid;

pub const HEADER: &str = "Idempotency-Key";

/// The header's UUID, or a new one; 400 if the header isn't a UUID.
pub fn request_id(headers: &axum::http::HeaderMap) -> Result<Uuid, crate::errors::ApiError> {
    let _ = headers;
    todo!()
}
