// doc/database.md#processed_requests

use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

use crate::error::Result;

pub fn find(c: &Connection, request_id: &Uuid) -> Result<Option<serde_json::Value>> {
    let text: Option<String> = c
        .query_row("SELECT response FROM processed_requests WHERE request_id = ?1", [request_id.to_string()], |r| r.get(0))
        .optional()?;
    Ok(text.and_then(|t| serde_json::from_str(&t).ok()))
}

pub fn record(c: &Connection, request_id: &Uuid, r#type: &str, response: &serde_json::Value, now_s: i64) -> Result<()> {
    c.execute(
        "INSERT INTO processed_requests (request_id, type, processed_at, response) VALUES (?1, ?2, ?3, ?4)",
        params![request_id.to_string(), r#type, now_s, response.to_string()],
    )?;
    Ok(())
}

// doc/programs/database_handler.md#startup
pub fn purge(c: &Connection, max_age_days: u32, now_s: i64) -> Result<usize> {
    Ok(c.execute("DELETE FROM processed_requests WHERE processed_at < ?1", [now_s - max_age_days as i64 * 86_400])?)
}
