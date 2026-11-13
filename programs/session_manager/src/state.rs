// doc/programs/session_manager.md#persistence-and-recovery

use serde::{Deserialize, Serialize};
use targetscope_common::model::{
    CurrentSession, CurrentShot, Device, InputMode, SessionSetup, SessionState, StagedPaperImage, TargetType,
};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoredSession {
    pub session: CurrentSession,
    pub save_request_id: Option<Uuid>,
    pub running_since_ms: Option<i64>,
    pub last_change_ms: i64,
    pub device: Option<Device>,
    pub target_type: Option<TargetType>,
    pub shots_version: u64,
    pub live_paper_offset: i64,
    pub last_live_paper: Option<i64>,
    pub live_active: bool,
    pub save_warnings: Vec<String>,
}

impl StoredSession {
    pub fn idle() -> Self {
        Self::new(SessionState::Idle, InputMode::Manual)
    }

    // doc/interfaces.md#currentsession
    pub fn new(state: SessionState, input_mode: InputMode) -> Self {
        StoredSession {
            session: CurrentSession {
                state,
                recovered_from: None,
                setup: SessionSetup {
                    input_mode,
                    shooter_id: None,
                    device_id: None,
                    distance_m: None,
                    target_type_id: None,
                    single_handed: false,
                    supported: false,
                },
                shots: Vec::new(),
                paper_no: 1,
                paper_images: Vec::new(),
                notes: None,
                analysis: None,
                analysis_outdated: false,
                started_at: None,
                duration_s: 0,
                calibration: None,
            },
            save_request_id: None,
            running_since_ms: None,
            last_change_ms: 0,
            device: None,
            target_type: None,
            shots_version: 0,
            live_paper_offset: 0,
            last_live_paper: None,
            live_active: false,
            save_warnings: Vec::new(),
        }
    }

    pub fn state(&self) -> SessionState {
        self.session.state
    }

    pub fn mode(&self) -> InputMode {
        self.session.setup.input_mode
    }

    // doc/programs/session_manager.md#live-analysis
    pub fn shots_changed(&mut self) {
        self.shots_version += 1;
        self.session.analysis_outdated = true;
    }

    pub fn push_shot(&mut self, shot: CurrentShot) {
        self.session.shots.push(shot);
        self.shots_changed();
    }

    // doc/programs/session_manager.md#target-papers
    pub fn add_paper_image(&mut self, paper_no: i64, staged_file: Option<String>) {
        if let Some(staged_file) = staged_file {
            self.session.paper_images.retain(|i| i.paper_no != paper_no);
            self.session.paper_images.push(StagedPaperImage { paper_no, staged_file });
        }
    }

    pub fn bullet_diameter_mm(&self) -> Option<f64> {
        self.device.as_ref().map(|d| d.bullet_diameter_mm)
    }

    // doc/interfaces.md#currentsession
    pub fn snapshot(&self, now_ms: i64) -> CurrentSession {
        let mut s = self.session.clone();
        if let Some(since) = self.running_since_ms {
            s.duration_s += (now_ms - since).max(0) / 1000;
        }
        s
    }

    // doc/programs/session_manager.md#transitions-and-actions
    pub fn stop_clock(&mut self, now_ms: i64) {
        if let Some(since) = self.running_since_ms.take() {
            self.session.duration_s += (now_ms - since).max(0) / 1000;
        }
    }
}
