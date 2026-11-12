//! States and the stored session (doc/programs/session_manager.md#state-machine,
//! #persistence-and-recovery).

use serde::{Deserialize, Serialize};
use targetscope_common::model::{CurrentSession, Device, TargetType};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum State {
    Idle,
    Setup,
    Running,
    Paused,
    Review,
    Recovered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputMode {
    Camera,
    Manual,
    Photo,
}

/// CurrentSession plus internal fields; stored as JSON under `session_manager:current`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredSession {
    pub session: CurrentSession,
    pub save_request_id: Option<Uuid>,
    pub running_since_ms: Option<i64>,
    pub device: Option<Device>,
    pub target_type: Option<TargetType>,
    /// Increases with every shot change; analysis results for older versions are dropped.
    pub shots_version: u64,
    pub live_paper_offset: i64,
    pub last_live_paper: Option<u32>,
    pub save_warnings: Vec<String>,
}

impl StoredSession {
    pub fn idle() -> Self {
        todo!()
    }
}
