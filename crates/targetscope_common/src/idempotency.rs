// doc/shared_libraries.md#idempotency
// doc/interfaces.md#duplicate-protection-idempotency

use redis::aio::ConnectionManager;
use uuid::Uuid;

use crate::bus::{Bus, BusError};

pub const TTL_SECONDS: u64 = 86_400;

#[derive(Clone)]
pub struct RedisIdempotencyStore {
    conn: ConnectionManager,
    program: String,
}

impl RedisIdempotencyStore {
    pub fn new(bus: &Bus) -> Self {
        RedisIdempotencyStore { conn: bus.connection(), program: bus.program().to_string() }
    }

    pub fn key(&self, id: &Uuid) -> String {
        format!("{}:processed:{id}", self.program)
    }

    pub async fn get(&self, id: &Uuid) -> Result<Option<serde_json::Value>, BusError> {
        let data: Option<String> = redis::cmd("GET").arg(self.key(id)).query_async(&mut self.conn.clone()).await?;
        data.map(|d| serde_json::from_str(&d).map_err(|e| BusError::Decode(e.to_string()))).transpose()
    }

    pub fn record(&self, pipe: &mut redis::Pipeline, id: &Uuid, response: &serde_json::Value) {
        pipe.cmd("SET").arg(self.key(id)).arg(response.to_string()).arg("EX").arg(TTL_SECONDS).ignore();
    }
}
