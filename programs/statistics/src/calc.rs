//! Steps 4-6: values per session (mean point, extreme spread, mean radius) and per group/period;
//! score_per_shot_avg = Σ score / Σ shot_count (weighted by shots); at most 5 000 shots, newest first.

use targetscope_common::model::{OffsetMm, Session, Shot, StatisticsValues};

pub const MAX_SHOTS: usize = 5_000;

pub struct SessionValues {
    pub mean_point: OffsetMm,
    /// Largest distance between two shots; 0 for one shot.
    pub extreme_spread_mm: f64,
    /// Mean of |p_j - m|
    pub mean_radius_mm: f64,
}

pub fn session_values(session: &Session) -> SessionValues {
    let _ = session;
    todo!()
}

pub fn aggregate(sessions: &[&Session]) -> StatisticsValues {
    let _ = sessions;
    todo!()
}

/// All shots of the group for the distribution plot, newest sessions first, at most MAX_SHOTS.
pub fn distribution_shots(sessions: &[&Session]) -> Vec<Shot> {
    let _ = sessions;
    todo!()
}
