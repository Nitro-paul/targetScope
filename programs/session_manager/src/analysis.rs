//! Live analysis (doc/programs/session_manager.md#live-analysis): only one analysis at a time;
//! if shots change meanwhile, one more runs afterwards with the newest shots.

use targetscope_common::model::{Shot, TargetType};

pub struct AnalysisRunner {
    pub running: Option<u64>,
    pub pending: Option<u64>,
}

impl AnalysisRunner {
    pub fn new() -> Self {
        Self {
            running: None,
            pending: None,
        }
    }

    /// Starts analyze_shots for this shots_version, or marks one as pending.
    pub fn request(
        &mut self,
        shots_version: u64,
        shots: Vec<Shot>,
        target: TargetType,
        bullet_diameter_mm: f64,
    ) {
        let _ = (shots_version, shots, target, bullet_diameter_mm);
        todo!()
    }
}

impl Default for AnalysisRunner {
    fn default() -> Self {
        Self::new()
    }
}
