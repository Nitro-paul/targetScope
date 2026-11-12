//! session_save (doc/programs/session_manager.md#saving): save_request_id, re-read device and
//! target type, current analysis, paper images, create_session (retry without missing images),
//! then delete the state and send session_saved.

use targetscope_common::bus::Bus;
use targetscope_common::messages::ServiceError;

use crate::state::StoredSession;

pub struct SaveOutcome {
    pub session_id: i64,
    pub warnings: Vec<String>,
}

pub async fn save(bus: &Bus, session: &mut StoredSession) -> Result<SaveOutcome, ServiceError> {
    let _ = (bus, session);
    todo!()
}
