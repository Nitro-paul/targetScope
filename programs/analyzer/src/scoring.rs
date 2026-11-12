//! Scoring (doc/programs/analyzer.md#definitions) and the grouping search
//! (doc/programs/analyzer.md#grouping-algorithm). No dependencies besides the shared model.

use targetscope_common::model::{AnalysisResult, Ring, Shot, TargetType};

/// Comparisons of distances use this tolerance (mm).
pub const EPSILON_MM: f64 = 1e-9;

/// Edge rule: s_i for the smallest i with d - b <= r_i, 0 if there is none.
/// `rings` ordered from the inside out.
pub fn shot_score(distance_mm: f64, rings: &[Ring], bullet_radius_mm: f64) -> i64 {
    let _ = (distance_mm, rings, bullet_radius_mm);
    todo!()
}

/// Shot scores, session score, grouping score and shift, mean point of impact.
pub fn analyze(shots: &[Shot], target: &TargetType, bullet_diameter_mm: f64) -> AnalysisResult {
    let _ = (shots, target, bullet_diameter_mm);
    todo!()
}

/// Two-stage grid search: coarse step h = min(0.5 mm, (r_1 + b) / 5) over the bounding box
/// extended by r_n + b, plus (0, 0) and the mean point; 10 best → fine grid ±h with step h/10.
/// Ties: smallest |c|, then c_x, then c_y. Returns (grouping_score, shift).
pub fn grouping(points: &[(f64, f64)], rings: &[Ring], bullet_radius_mm: f64) -> (i64, (f64, f64)) {
    let _ = (points, rings, bullet_radius_mm);
    todo!()
}
