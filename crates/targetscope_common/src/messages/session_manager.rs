//! Messages of `session_manager` (doc/interfaces.md#session_manager).
//! Source of truth: `schemas/session_manager/`.

use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::Request;
use crate::model::{Corners, CurrentSession};

pub const STREAM: &str = "session_manager:requests";

/// Requests understood by `session_manager`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum SessionManagerRequest {
    Ping {},
    SessionGet(SessionGet),
    SessionSetup(SessionSetup),
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

/// Events on `session_manager:events`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum SessionManagerEvent {
    SessionChanged(SessionChanged),
    SessionSaved(SessionSaved),
}

/// `session_get` request (doc/interfaces.md#session_get, schemas/session_manager/session_get.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionGet {}

/// `session_get_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionGetResponse {
    pub session: CurrentSession,
}

impl Request for SessionGet {
    type Response = SessionGetResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}

/// `session_setup` request (doc/interfaces.md#session_setup, schemas/session_manager/session_setup.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionSetup {
    // TODO: fields
}

/// `session_setup_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionSetupResponse {
    pub session: CurrentSession,
}

impl Request for SessionSetup {
    type Response = SessionSetupResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}

/// `session_update_setup` request (doc/interfaces.md#session_update_setup, schemas/session_manager/session_update_setup.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionUpdateSetup {
    // TODO: fields
}

/// `session_update_setup_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionUpdateSetupResponse {
    pub session: CurrentSession,
}

impl Request for SessionUpdateSetup {
    type Response = SessionUpdateSetupResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(30);
}

/// `session_calibrate` request (doc/interfaces.md#session_calibrate, schemas/session_manager/session_calibrate.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionCalibrate {
    pub corners: Corners,
}

/// `session_calibrate_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionCalibrateResponse {
    pub session: CurrentSession,
}

impl Request for SessionCalibrate {
    type Response = SessionCalibrateResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(30);
}

/// `session_start` request (doc/interfaces.md#session_start, schemas/session_manager/session_start.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionStart {}

/// `session_start_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionStartResponse {
    pub session: CurrentSession,
}

impl Request for SessionStart {
    type Response = SessionStartResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(30);
}

/// `session_upload_photo` request (doc/interfaces.md#session_upload_photo, schemas/session_manager/session_upload_photo.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionUploadPhoto {
    // TODO: fields
}

/// `session_upload_photo_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionUploadPhotoResponse {
    pub session: CurrentSession,
}

impl Request for SessionUploadPhoto {
    type Response = SessionUploadPhotoResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(90);
}

/// `session_pause` request (doc/interfaces.md#session_pause, schemas/session_manager/session_pause.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionPause {}

/// `session_pause_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionPauseResponse {
    pub session: CurrentSession,
}

impl Request for SessionPause {
    type Response = SessionPauseResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(30);
}

/// `session_resume` request (doc/interfaces.md#session_resume, schemas/session_manager/session_resume.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionResume {}

/// `session_resume_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionResumeResponse {
    pub session: CurrentSession,
}

impl Request for SessionResume {
    type Response = SessionResumeResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(30);
}

/// `session_add_shot` request (doc/interfaces.md#session_add_shot, schemas/session_manager/session_add_shot.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionAddShot {
    // TODO: fields
}

/// `session_add_shot_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionAddShotResponse {
    pub session: CurrentSession,
}

impl Request for SessionAddShot {
    type Response = SessionAddShotResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}

/// `session_update_shot` request (doc/interfaces.md#session_update_shot, schemas/session_manager/session_update_shot.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionUpdateShot {
    // TODO: fields
}

/// `session_update_shot_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionUpdateShotResponse {
    pub session: CurrentSession,
}

impl Request for SessionUpdateShot {
    type Response = SessionUpdateShotResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}

/// `session_delete_shot` request (doc/interfaces.md#session_delete_shot, schemas/session_manager/session_delete_shot.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionDeleteShot {
    pub index: u32,
}

/// `session_delete_shot_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionDeleteShotResponse {
    pub session: CurrentSession,
}

impl Request for SessionDeleteShot {
    type Response = SessionDeleteShotResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(30);
}

/// `session_new_paper` request (doc/interfaces.md#session_new_paper, schemas/session_manager/session_new_paper.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionNewPaper {}

/// `session_new_paper_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionNewPaperResponse {
    pub session: CurrentSession,
}

impl Request for SessionNewPaper {
    type Response = SessionNewPaperResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(30);
}

/// `session_set_notes` request (doc/interfaces.md#session_set_notes, schemas/session_manager/session_set_notes.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionSetNotes {
    pub notes: Option<String>,
}

/// `session_set_notes_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionSetNotesResponse {
    pub session: CurrentSession,
}

impl Request for SessionSetNotes {
    type Response = SessionSetNotesResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}

/// `session_stop` request (doc/interfaces.md#session_stop, schemas/session_manager/session_stop.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionStop {}

/// `session_stop_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionStopResponse {
    pub session: CurrentSession,
}

impl Request for SessionStop {
    type Response = SessionStopResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}

/// `session_save` request (doc/interfaces.md#session_save, schemas/session_manager/session_save.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionSave {}

/// `session_save_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionSaveResponse {
    pub session: CurrentSession,
    pub session_id: i64,
    pub warnings: Vec<String>,
}

impl Request for SessionSave {
    type Response = SessionSaveResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(30);
}

/// `session_cancel` request (doc/interfaces.md#session_cancel, schemas/session_manager/session_cancel.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionCancel {}

/// `session_cancel_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionCancelResponse {
    pub session: CurrentSession,
}

impl Request for SessionCancel {
    type Response = SessionCancelResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(30);
}

/// `session_changed` event (doc/interfaces.md#session_changed, schemas/session_manager/session_changed.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionChanged {
    pub session: CurrentSession,
}

/// `session_saved` event (doc/interfaces.md#session_saved, schemas/session_manager/session_saved.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionSaved {
    pub session_id: i64,
}
