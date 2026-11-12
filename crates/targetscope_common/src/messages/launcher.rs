//! Messages of `launcher` (doc/interfaces.md#launcher).
//! Source of truth: `schemas/launcher/`.

use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::Request;

pub const STREAM: &str = "launcher:requests";

/// Requests understood by `launcher`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum LauncherRequest {
    Ping {},
    LauncherStatus(LauncherStatus),
    RestartProgram(RestartProgram),
}

/// Events on `launcher:events`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum LauncherEvent {
    ProgramStateChanged(ProgramStateChanged),
}

/// `launcher_status` request (doc/interfaces.md#launcher_status, schemas/launcher/launcher_status.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LauncherStatus {}

/// `launcher_status_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LauncherStatusResponse {
    // TODO: fields
}

impl Request for LauncherStatus {
    type Response = LauncherStatusResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}

/// `restart_program` request (doc/interfaces.md#restart_program, schemas/launcher/restart_program.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RestartProgram {
    pub name: String,
}

/// `restart_program_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RestartProgramResponse {
    pub name: String,
}

impl Request for RestartProgram {
    type Response = RestartProgramResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}

/// `program_state_changed` event (doc/interfaces.md#program_state_changed, schemas/launcher/program_state_changed.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProgramStateChanged {
    // TODO: fields
}
