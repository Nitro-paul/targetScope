//! Allowed requests per state, guards and actions
//! (doc/programs/session_manager.md#transitions-and-actions).
//! A request not allowed in the state → invalid_state; a failed guard → validation_failed.

use targetscope_common::messages::session_manager::SessionManagerRequest;
use targetscope_common::messages::ServiceError;

use crate::state::State;

/// Is the request allowed in this state at all?
pub fn is_allowed(state: State, req: &SessionManagerRequest) -> bool {
    let _ = (state, req);
    todo!()
}

/// Guard of a transition, e.g. session_start without device or target type.
pub fn check_guard(
    session: &crate::state::StoredSession,
    req: &SessionManagerRequest,
) -> Result<(), ServiceError> {
    let _ = (session, req);
    todo!()
}
