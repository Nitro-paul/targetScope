// doc/interfaces.md#session_manager

use serde::{Deserialize, Deserializer, Serialize};

use super::{request, Ping};
use crate::model::{Corners, CurrentSession, InputMode};

pub const PROGRAM: &str = "session_manager";
pub const STREAM: &str = "session_manager:requests";

// doc/shared_libraries.md#messages
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum SessionManagerRequest {
    Ping(Ping),
    SessionGet(SessionGet),
    SessionSetup(SessionSetupRequest),
    SessionUpdateSetup(SessionUpdateSetup),
    SessionCalibrate(SessionCalibrate),
    SessionStart(SessionStart),
    SessionUploadPhoto(SessionUploadPhoto),
    SessionPause(SessionPause),
    SessionResume(SessionResume),
    SessionAddShot(SessionAddShot),
    SessionUpdateShot(SessionUpdateShot),
    SessionDeleteShot(SessionDeleteShot),
    SessionNewPaper(SessionNewPaper),
    SessionSetNotes(SessionSetNotes),
    SessionStop(SessionStop),
    SessionSave(SessionSave),
    SessionCancel(SessionCancel),
}

// doc/interfaces.md#events-session_managerevents
// doc/shared_libraries.md#messages
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum SessionManagerEvent {
    SessionChanged(SessionResponse),
    SessionSaved(SessionSaved),
}

// doc/interfaces.md#session_manager
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionResponse {
    pub session: CurrentSession,
}

macro_rules! empty_request {
    ($name:ident, $type:literal, $secs:expr) => {
        #[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
        pub struct $name {}
        request!($name, SessionResponse, $type, $secs);
    };
}

empty_request!(SessionGet, "session_get", 5);
empty_request!(SessionStart, "session_start", 30);
empty_request!(SessionPause, "session_pause", 30);
empty_request!(SessionResume, "session_resume", 30);
empty_request!(SessionNewPaper, "session_new_paper", 30);
empty_request!(SessionStop, "session_stop", 30);
empty_request!(SessionCancel, "session_cancel", 30);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionSetupRequest {
    pub input_mode: InputMode,
}
request!(SessionSetupRequest, SessionResponse, "session_setup", 5);

// doc/interfaces.md#session_manager
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionUpdateSetup {
    #[serde(default, deserialize_with = "present", skip_serializing_if = "Option::is_none")]
    pub shooter_id: Option<Option<i64>>,
    #[serde(default, deserialize_with = "present", skip_serializing_if = "Option::is_none")]
    pub device_id: Option<Option<i64>>,
    #[serde(default, deserialize_with = "crate::check::opt_opt_count", skip_serializing_if = "Option::is_none")]
    pub distance_m: Option<Option<i64>>,
    #[serde(default, deserialize_with = "present", skip_serializing_if = "Option::is_none")]
    pub target_type_id: Option<Option<i64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub single_handed: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supported: Option<bool>,
}
request!(SessionUpdateSetup, SessionResponse, "session_update_setup", 30);

fn present<'de, D, T>(d: D) -> Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(d).map(Some)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionCalibrate {
    pub corners: Corners,
}
request!(SessionCalibrate, SessionResponse, "session_calibrate", 30);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionUploadPhoto {
    pub staged_file: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub corners: Option<Corners>,
}
request!(SessionUploadPhoto, SessionResponse, "session_upload_photo", 90);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionAddShot {
    #[serde(deserialize_with = "crate::check::angle_turns")]
    pub angle_turns: f64,
    #[serde(deserialize_with = "crate::check::non_negative")]
    pub radius_mm: f64,
    #[serde(default, deserialize_with = "crate::check::opt_number", skip_serializing_if = "Option::is_none")]
    pub paper_no: Option<i64>,
}
request!(SessionAddShot, SessionResponse, "session_add_shot", 5);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionUpdateShot {
    pub index: i64,
    #[serde(deserialize_with = "crate::check::angle_turns")]
    pub angle_turns: f64,
    #[serde(deserialize_with = "crate::check::non_negative")]
    pub radius_mm: f64,
}
request!(SessionUpdateShot, SessionResponse, "session_update_shot", 5);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionDeleteShot {
    pub index: i64,
}
request!(SessionDeleteShot, SessionResponse, "session_delete_shot", 30);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionSetNotes {
    #[serde(default, deserialize_with = "crate::check::notes")]
    pub notes: Option<String>,
}
request!(SessionSetNotes, SessionResponse, "session_set_notes", 5);

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionSave {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionSaveResponse {
    pub session: CurrentSession,
    pub session_id: i64,
    pub warnings: Vec<String>,
}
request!(SessionSave, SessionSaveResponse, "session_save", 30);

// doc/interfaces.md#session_saved
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionSaved {
    pub session_id: i64,
}
