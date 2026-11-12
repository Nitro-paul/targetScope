// doc/programs/statistics.md#calculation-flow-compute_statistics
// doc/interfaces.md#statisticsvalues

use targetscope_common::geometry::Point;
use targetscope_common::model::{OffsetMm, Session, Shot, StatisticsValues};

pub const MAX_SHOTS: usize = 5_000;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SessionValues {
    pub mean_point: Point,
    pub extreme_spread_mm: f64,
    pub mean_radius_mm: f64,
}

pub fn session_values(shots: &[Shot]) -> SessionValues {
    let points: Vec<Point> = shots.iter().map(|s| Point::from(*s)).collect();
    let n = points.len().max(1) as f64;
    let mean = Point { x: points.iter().map(|p| p.x).sum::<f64>() / n, y: points.iter().map(|p| p.y).sum::<f64>() / n };
    let mut spread = 0.0f64;
    for (i, a) in points.iter().enumerate() {
        for b in &points[i + 1..] {
            spread = spread.max(a.distance(b));
        }
    }
    let mean_radius = points.iter().map(|p| p.distance(&mean)).sum::<f64>() / n;
    SessionValues { mean_point: mean, extreme_spread_mm: spread, mean_radius_mm: mean_radius }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SessionSummary {
    pub score: i64,
    pub grouping_score: i64,
    pub shot_count: i64,
    pub values: SessionValues,
}

pub fn summarize(s: &Session) -> SessionSummary {
    let shots: Vec<Shot> = s.shots.iter().map(|x| x.shot()).collect();
    SessionSummary { score: s.score, grouping_score: s.grouping_score, shot_count: s.shot_count, values: session_values(&shots) }
}

pub fn aggregate(sessions: &[SessionSummary]) -> StatisticsValues {
    let n = sessions.len().max(1) as f64;
    let shots: i64 = sessions.iter().map(|s| s.shot_count).sum();
    let per_shot = |total: i64| if shots > 0 { total as f64 / shots as f64 } else { 0.0 };
    StatisticsValues {
        session_count: sessions.len() as i64,
        shot_count: shots,
        score_avg: sessions.iter().map(|s| s.score as f64).sum::<f64>() / n,
        score_min: sessions.iter().map(|s| s.score).min().unwrap_or(0),
        score_max: sessions.iter().map(|s| s.score).max().unwrap_or(0),
        score_per_shot_avg: per_shot(sessions.iter().map(|s| s.score).sum()),
        grouping_score_per_shot_avg: per_shot(sessions.iter().map(|s| s.grouping_score).sum()),
        extreme_spread_mm_avg: sessions.iter().map(|s| s.values.extreme_spread_mm).sum::<f64>() / n,
        mean_radius_mm_avg: sessions.iter().map(|s| s.values.mean_radius_mm).sum::<f64>() / n,
        mean_point_avg: OffsetMm {
            x_mm: sessions.iter().map(|s| s.values.mean_point.x).sum::<f64>() / n,
            y_mm: sessions.iter().map(|s| s.values.mean_point.y).sum::<f64>() / n,
        },
    }
}

// doc/interfaces.md#statisticsgroup
pub fn distribution_shots(sessions: &[&Session]) -> Vec<Shot> {
    let mut newest: Vec<&&Session> = sessions.iter().collect();
    newest.sort_by(|a, b| b.saved_at.cmp(&a.saved_at).then(b.id.cmp(&a.id)));
    newest.iter().flat_map(|s| s.shots.iter().map(|x| x.shot())).take(MAX_SHOTS).collect()
}
