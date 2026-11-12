// doc/programs/web_server.md#rest-api

use axum::http::HeaderMap;
use uuid::Uuid;

use crate::errors::ApiError;

pub const HEADER: &str = "idempotency-key";

pub fn request_id(headers: &HeaderMap) -> Result<Uuid, ApiError> {
    match headers.get(HEADER) {
        None => Ok(Uuid::new_v4()),
        Some(v) => v
            .to_str()
            .ok()
            .and_then(|s| Uuid::parse_str(s.trim()).ok())
            .ok_or_else(|| ApiError::validation("Idempotency-Key must be a UUID")),
    }
}
