// doc/programs/analyzer.md#definitions

use targetscope_common::geometry::Point;
use targetscope_common::model::{AnalysisResult, OffsetMm, Ring, Shot, TargetType};
use targetscope_common::targets::generate_rings;

// doc/programs/analyzer.md#shot-score-edge-rule
pub const EPSILON_MM: f64 = 1e-9;

// doc/programs/analyzer.md#shot-score-edge-rule
pub fn shot_score(distance_mm: f64, rings: &[Ring], bullet_radius_mm: f64) -> i64 {
    let edge = distance_mm - bullet_radius_mm - EPSILON_MM;
    let i = rings.partition_point(|r| r.radius_mm < edge);
    rings.get(i).map_or(0, |r| r.score)
}

pub fn analyze(shots: &[Shot], target: &TargetType, bullet_diameter_mm: f64) -> AnalysisResult {
    let rings = generate_rings(target);
    let b = bullet_diameter_mm / 2.0;
    let points: Vec<Point> = shots.iter().map(|s| Point::from(*s)).collect();
    let shot_scores: Vec<i64> = points.iter().map(|p| shot_score(p.length(), &rings, b)).collect();
    let score = shot_scores.iter().sum();
    if points.is_empty() {
        return AnalysisResult {
            shot_scores,
            score,
            grouping_score: 0,
            grouping_shift: OffsetMm::default(),
            mean_point: None,
        };
    }
    // doc/programs/analyzer.md#mean-point-of-impact
    let n = points.len() as f64;
    let mean = Point { x: points.iter().map(|p| p.x).sum::<f64>() / n, y: points.iter().map(|p| p.y).sum::<f64>() / n };
    let (grouping_score, shift) = grouping(&points, &rings, b, mean);
    AnalysisResult {
        shot_scores,
        score,
        grouping_score,
        grouping_shift: OffsetMm { x_mm: shift.x, y_mm: shift.y },
        mean_point: Some(OffsetMm { x_mm: mean.x, y_mm: mean.y }),
    }
}

// doc/programs/analyzer.md#grouping-score
pub fn total_score(points: &[Point], c: Point, rings: &[Ring], b: f64) -> i64 {
    points.iter().map(|p| shot_score(p.distance(&c), rings, b)).sum()
}

#[derive(Debug, Clone, Copy)]
struct Candidate {
    score: i64,
    c: Point,
}

// doc/programs/analyzer.md#grouping-algorithm
fn better(a: &Candidate, b: &Candidate) -> std::cmp::Ordering {
    b.score
        .cmp(&a.score)
        .then(a.c.length().total_cmp(&b.c.length()))
        .then(a.c.x.total_cmp(&b.c.x))
        .then(a.c.y.total_cmp(&b.c.y))
}

// doc/programs/analyzer.md#grouping-algorithm
pub fn grouping(points: &[Point], rings: &[Ring], b: f64, mean: Point) -> (i64, Point) {
    let (Some(first), Some(last)) = (rings.first(), rings.last()) else {
        return (0, Point { x: 0.0, y: 0.0 });
    };
    let reach = last.radius_mm + b;
    let min_x = points.iter().map(|p| p.x).fold(f64::INFINITY, f64::min) - reach;
    let max_x = points.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max) + reach;
    let min_y = points.iter().map(|p| p.y).fold(f64::INFINITY, f64::min) - reach;
    let max_y = points.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max) + reach;
    let h = 0.5f64.min((first.radius_mm + b) / 5.0);

    let eval = |c: Point| Candidate { score: total_score(points, c, rings, b), c };
    let mut coarse = vec![eval(Point { x: 0.0, y: 0.0 }), eval(mean)];
    let nx = ((max_x - min_x) / h).ceil() as i64;
    let ny = ((max_y - min_y) / h).ceil() as i64;
    for i in 0..=nx {
        for j in 0..=ny {
            coarse.push(eval(Point { x: min_x + i as f64 * h, y: min_y + j as f64 * h }));
        }
    }
    coarse.sort_by(better);
    coarse.truncate(10);

    let mut best = coarse[0];
    let fine = h / 10.0;
    for start in &coarse {
        for i in -10..=10 {
            for j in -10..=10 {
                let cand = eval(Point { x: start.c.x + i as f64 * fine, y: start.c.y + j as f64 * fine });
                if better(&cand, &best) == std::cmp::Ordering::Less {
                    best = cand;
                }
            }
        }
    }
    (best.score, best.c)
}
