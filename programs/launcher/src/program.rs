// doc/programs/launcher.md#internal-design

use std::collections::VecDeque;
use std::time::{Duration, Instant, SystemTime};

use targetscope_common::clock::unix_ms;
use targetscope_common::messages::launcher::{ProgramState, ProgramStatus};

use crate::config::ProgramSpec;

pub struct ProgramHandle {
    pub spec: ProgramSpec,
    pub state: ProgramState,
    pub pid: Option<u32>,
    pub generation: u64,
    pub restarts: VecDeque<Instant>,
    pub restart_count: i64,
    pub backoff: Duration,
    pub since: SystemTime,
    pub started: Option<Instant>,
    pub restart_requested: bool,
    pub ever_started: bool,
}

impl ProgramHandle {
    pub fn new(spec: ProgramSpec, backoff: Duration) -> Self {
        ProgramHandle {
            spec,
            state: ProgramState::Stopped,
            pid: None,
            generation: 0,
            restarts: VecDeque::new(),
            restart_count: 0,
            backoff,
            since: SystemTime::now(),
            started: None,
            restart_requested: false,
            ever_started: false,
        }
    }

    // doc/interfaces.md#launcher_status
    pub fn status(&self) -> ProgramStatus {
        ProgramStatus {
            name: self.spec.name.clone(),
            state: self.state,
            pid: self.pid.map(|p| p as i64),
            restarts: self.restart_count,
            since_ms: unix_ms(self.since),
        }
    }
}
