// doc/programs/session_manager.md#live-analysis

use targetscope_common::bus::Bus;
use targetscope_common::messages::analyzer::AnalyzeShots;
use tokio::sync::mpsc;

use crate::actor::Input;
use crate::state::StoredSession;

#[derive(Default)]
pub struct AnalysisRunner {
    running: Option<u64>,
    pending: bool,
}

impl AnalysisRunner {
    pub fn request(&mut self, s: &StoredSession, bus: &Bus, tx: &mpsc::Sender<Input>) {
        if self.running.is_some() {
            self.pending = true;
            return;
        }
        let (Some(target_type), Some(bullet_diameter_mm)) = (s.target_type.clone(), s.bullet_diameter_mm()) else {
            return;
        };
        let version = s.shots_version;
        self.running = Some(version);
        let req = AnalyzeShots { shots: s.session.shots.iter().map(|x| x.shot()).collect(), target_type, bullet_diameter_mm };
        let (bus, tx) = (bus.clone(), tx.clone());
        tokio::spawn(async move {
            let result = bus.request(req).await;
            let _ = tx.send(Input::Analysis { shots_version: version, result }).await;
        });
    }

    // doc/programs/session_manager.md#live-analysis
    pub fn finished(&mut self) -> bool {
        self.running = None;
        std::mem::take(&mut self.pending)
    }
}
