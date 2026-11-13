// doc/shared_libraries.md#target-types
// doc/interfaces.md#targettype

use crate::model::{Ring, TargetType};

pub fn generate_rings(t: &TargetType) -> Vec<Ring> {
    let anchors = [(t.inner_white_score, t.inner_white_radius_mm), (t.black_score, t.black_radius_mm), (t.outer_score, t.outer_radius_mm)];
    let mut rings = Vec::new();
    let mut score = t.inner_white_score;
    while score >= t.outer_score && score >= 1 {
        rings.push(Ring { score, radius_mm: ring_radius(&anchors, score) });
        score -= 1;
    }
    rings
}

fn ring_radius(anchors: &[(i64, f64); 3], s: i64) -> f64 {
    if let Some(&(_, r)) = anchors.iter().find(|(score, _)| *score == s) {
        return r;
    }
    let (a, b) = if s > anchors[1].0 { (anchors[0], anchors[1]) } else { (anchors[1], anchors[2]) };
    let (s_a, r_a) = (a.0 as f64, a.1);
    let (s_b, r_b) = (b.0 as f64, b.1);
    r_a + (s_a - s as f64) * (r_b - r_a) / (s_a - s_b)
}

pub fn validate_target_type(t: &TargetType) -> Result<(), String> {
    let finite =
        [t.paper_width_mm, t.paper_height_mm, t.center_x_mm, t.center_y_mm, t.inner_white_radius_mm, t.black_radius_mm, t.outer_radius_mm];
    if finite.iter().any(|v| !v.is_finite()) {
        return Err("all sizes are finite numbers".into());
    }
    let rules: [(bool, &str); 11] = [
        (t.paper_width_mm > 0.0 && t.paper_height_mm > 0.0, "paper size > 0"),
        (t.inner_white_score > t.black_score, "inner_white_score > black_score"),
        (t.black_score > t.outer_score, "black_score > outer_score"),
        (t.outer_score >= 1, "outer_score >= 1"),
        (t.inner_white_radius_mm > 0.0, "inner_white_radius_mm > 0"),
        (t.inner_white_radius_mm < t.black_radius_mm, "inner_white_radius_mm < black_radius_mm"),
        (t.black_radius_mm < t.outer_radius_mm, "black_radius_mm < outer_radius_mm"),
        (t.center_x_mm - t.outer_radius_mm >= 0.0, "outermost ring on the paper (left edge)"),
        (t.center_x_mm + t.outer_radius_mm <= t.paper_width_mm, "outermost ring on the paper (right edge)"),
        (t.center_y_mm - t.outer_radius_mm >= 0.0, "outermost ring on the paper (top edge)"),
        (t.center_y_mm + t.outer_radius_mm <= t.paper_height_mm, "outermost ring on the paper (bottom edge)"),
    ];
    match rules.iter().find(|(ok, _)| !ok) {
        Some((_, rule)) => Err((*rule).to_string()),
        None => Ok(()),
    }
}
