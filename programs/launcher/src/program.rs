//! State of one supervised program (doc/programs/launcher.md#internal-design).

use std::collections::VecDeque;
use std::time::{Duration, Instant, SystemTime};

use serde::Serialize;

use crate::config::ProgramSpec;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProgramState {
    Starting,
    Running,
    Backoff,
    Failed,
    Stopped,
}

pub struct ProgramHandle {
    pub spec: ProgramSpec,
    pub state: ProgramState,
    pub child: Option<tokio::process::Child>,
    /// Restart times within window_s
    pub restarts: VecDeque<Instant>,
    pub backoff: Duration,
    pub since: SystemTime,
}

/// One entry of `launcher_status_response.programs` and payload of `program_state_changed`.
#[derive(Debug, Clone, Serialize)]
pub struct ProgramStatus {
    pub name: String,
    pub state: ProgramState,
    pub pid: Option<u32>,
    pub restarts: u32,
    pub since_ms: i64,
}

impl ProgramHandle {
    pub fn status(&self) -> ProgramStatus {
        todo!()
    }
}
