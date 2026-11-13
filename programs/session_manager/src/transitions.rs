// doc/programs/session_manager.md#state-machine
// doc/testing/session_manager.md#state-machine-integration

use targetscope_common::messages::session_manager::SessionManagerRequest as R;
use targetscope_common::model::{InputMode as M, SessionState as S};

pub fn is_allowed(state: S, mode: M, req: &R) -> bool {
    let camera = mode == M::Camera;
    let manual = mode == M::Manual;
    let photo = mode == M::Photo;
    match req {
        R::Ping(_) | R::SessionGet(_) => true,
        R::SessionSetup(_) => state == S::Idle,
        R::SessionUpdateSetup(_) => state == S::Setup,
        R::SessionCalibrate(_) => matches!(state, S::Setup | S::Paused) && camera,
        R::SessionStart(_) => state == S::Setup && (camera || manual),
        R::SessionUploadPhoto(_) => matches!(state, S::Setup | S::Review) && photo,
        R::SessionPause(_) => state == S::Running,
        R::SessionResume(_) => state == S::Paused,
        R::SessionAddShot(_) | R::SessionUpdateShot(_) => (state == S::Running && manual) || state == S::Review,
        R::SessionDeleteShot(_) => matches!(state, S::Running | S::Paused | S::Review),
        R::SessionNewPaper(_) => matches!(state, S::Running | S::Paused) && (camera || manual),
        R::SessionSetNotes(_) => state != S::Idle,
        R::SessionStop(_) => matches!(state, S::Running | S::Paused),
        R::SessionSave(_) => matches!(state, S::Review | S::Recovered),
        R::SessionCancel(_) => state != S::Idle,
    }
}

pub fn changes_state(req: &R) -> bool {
    !matches!(req, R::Ping(_) | R::SessionGet(_))
}
