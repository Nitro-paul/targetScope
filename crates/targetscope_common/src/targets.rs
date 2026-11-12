//! Ring generation and target type validation (doc/shared_libraries.md#target-types).
//! The same two functions exist in targetscope_common_cpp and the frontend; all are tested
//! with `schemas/examples/target_types/*.json`.

use crate::model::{Ring, TargetType};

/// One ring per score from inner_white_score down to outer_score, radii spaced equally
/// between neighboring anchors: r(s) = r_a + (s_a - s) · (r_b - r_a) / (s_a - s_b).
pub fn generate_rings(t: &TargetType) -> Vec<Ring> {
    let _ = t;
    todo!()
}

/// All TargetType rules: radii increasing, scores decreasing, outermost ring on the paper.
pub fn validate_target_type(t: &TargetType) -> Result<(), String> {
    let _ = t;
    todo!()
}
