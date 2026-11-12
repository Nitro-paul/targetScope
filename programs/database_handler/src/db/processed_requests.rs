//! Duplicate protection: processed request ids with their responses.

use rusqlite::Connection;
use uuid::Uuid;

use crate::error::HandlerError;

/// Stored response payload if request_id was already processed.
pub fn find(c: &Connection, request_id: &Uuid) -> Result<Option<serde_json::Value>, HandlerError> {
    let _ = (c, request_id);
    todo!()
}

/// Must be called inside the transaction of the write it records.
pub fn record(
    c: &Connection,
    request_id: &Uuid,
    r#type: &str,
    response: &serde_json::Value,
) -> Result<(), HandlerError> {
    let _ = (c, request_id, r#type, response);
    todo!()
}

/// Deletes entries older than max_age_days; returns the number deleted.
pub fn purge(c: &Connection, max_age_days: u32) -> Result<usize, HandlerError> {
    let _ = (c, max_age_days);
    todo!()
}
