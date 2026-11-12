// doc/programs/session_manager.md#persistence-and-recovery

use redis::aio::ConnectionManager;
use serde_json::Value;
use targetscope_common::bus::BusError;
use targetscope_common::failpoints;
use targetscope_common::idempotency::RedisIdempotencyStore;
use targetscope_common::model::SessionState;
use uuid::Uuid;

use crate::state::StoredSession;

pub const CURRENT_KEY: &str = "session_manager:current";

pub async fn load(conn: &mut ConnectionManager) -> Result<Option<StoredSession>, BusError> {
    let data: Option<String> = redis::cmd("GET").arg(CURRENT_KEY).query_async(conn).await?;
    match data {
        None => Ok(None),
        Some(d) => serde_json::from_str(&d).map(Some).map_err(|e| BusError::Decode(e.to_string())),
    }
}

// doc/programs/session_manager.md#persistence-and-recovery
pub async fn commit(
    conn: &mut ConnectionManager,
    idem: &RedisIdempotencyStore,
    state: &StoredSession,
    record: Option<(Uuid, &Value)>,
) -> Result<(), BusError> {
    let mut pipe = redis::pipe();
    pipe.atomic();
    if state.state() == SessionState::Idle {
        pipe.cmd("DEL").arg(CURRENT_KEY).ignore();
    } else {
        let json = serde_json::to_string(state).map_err(|e| BusError::Decode(e.to_string()))?;
        pipe.cmd("SET").arg(CURRENT_KEY).arg(json).ignore();
    }
    if let Some((id, response)) = record {
        idem.record(&mut pipe, &id, response);
    }
    if failpoints::hit("sm.before_exec") {
        return Err(BusError::Decode("failpoint sm.before_exec".into()));
    }
    let () = pipe.query_async(conn).await?;
    Ok(())
}
