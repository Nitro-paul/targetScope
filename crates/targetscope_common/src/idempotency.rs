//! Duplicate protection in Redis (doc/shared_libraries.md#idempotency).
//! Used by session_manager and launcher; database_handler uses its own SQLite table.

use uuid::Uuid;

use crate::bus::BusError;

/// Redis key `<program>:processed:<request_id>` with the response, TTL 24 h.
pub struct RedisIdempotencyStore {
    // TODO: connection, program name
}

pub const TTL_SECONDS: u64 = 86_400;

impl RedisIdempotencyStore {
    pub async fn get(&self, id: &Uuid) -> Result<Option<serde_json::Value>, BusError> {
        let _ = id;
        todo!()
    }

    /// Adds `SET key value EX 86400` to an existing MULTI pipeline.
    pub fn record(&self, pipe: &mut redis::Pipeline, id: &Uuid, response: &serde_json::Value) {
        let _ = (pipe, id, response);
        todo!()
    }
}
