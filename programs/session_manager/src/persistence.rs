//! `session_manager:current` in Redis and recovery after a restart
//! (doc/programs/session_manager.md#persistence-and-recovery).

use targetscope_common::bus::{Bus, BusError};

use crate::state::StoredSession;

pub const CURRENT_KEY: &str = "session_manager:current";

/// Reads the stored session; a non-idle one becomes `recovered` (recovered_from = stored state).
pub async fn load(bus: &Bus) -> Result<StoredSession, BusError> {
    let _ = bus;
    todo!()
}
