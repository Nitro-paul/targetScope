// doc/interfaces.md#launcher

use serde::{Deserialize, Serialize};

use super::{request, Ping};

pub const PROGRAM: &str = "launcher";
pub const STREAM: &str = "launcher:requests";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum LauncherRequest {
    Ping(Ping),
    LauncherStatus(LauncherStatus),
    RestartProgram(RestartProgram),
}

// doc/interfaces.md#events-launcherevents
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum LauncherEvent {
    ProgramStateChanged(ProgramStatus),
}

// doc/interfaces.md#launcher_status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProgramState {
    Starting,
    Running,
    Backoff,
    Failed,
    Stopped,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProgramStatus {
    pub name: String,
    pub state: ProgramState,
    pub pid: Option<i64>,
    pub restarts: i64,
    pub since_ms: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LauncherStatus {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LauncherStatusResponse {
    pub programs: Vec<ProgramStatus>,
}
request!(LauncherStatus, LauncherStatusResponse, "launcher_status", 5);

// doc/interfaces.md#restart_program
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestartProgram {
    pub name: String,
}
request!(RestartProgram, RestartProgram, "restart_program", 5);
