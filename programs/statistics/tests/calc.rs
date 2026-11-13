// doc/testing/statistics.md#values-per-session-and-group-unit

use statistics::calc::{aggregate, session_values, SessionSummary, SessionValues};
use targetscope_common::geometry::Point;
use targetscope_common::model::Shot;

fn shots(points: &[(f64, f64)]) -> Vec<Shot> {
    points.iter().map(|(x, y)| Shot::from(Point { x: *x, y: *y })).collect()
}

fn near(a: f64, b: f64, tol: f64) -> bool {
    (a - b).abs() <= tol
}

fn summary(score: i64, grouping: i64, shots: i64, spread: f64, radius: f64, mean: (f64, f64)) -> SessionSummary {
    SessionSummary {
        score,
        grouping_score: grouping,
        shot_count: shots,
        values: SessionValues { mean_point: Point { x: mean.0, y: mean.1 }, extreme_spread_mm: spread, mean_radius_mm: radius },
    }
}

// ST-CAL-01
#[test]
fn st_cal_01_extreme_spread() {
    assert!(near(session_values(&shots(&[(0.0, 0.0), (6.0, 8.0), (0.0, 8.0)])).extreme_spread_mm, 10.0, 1e-9));
}

// ST-CAL-02
#[test]
fn st_cal_02_extreme_spread_of_one_shot() {
    assert_eq!(session_values(&shots(&[(3.0, 4.0)])).extreme_spread_mm, 0.0);
}

// ST-CAL-03
#[test]
fn st_cal_03_mean_radius() {
    let v = session_values(&shots(&[(-1.0, 0.0), (1.0, 0.0)]));
    assert!(near(v.mean_point.x, 0.0, 1e-9) && near(v.mean_point.y, 0.0, 1e-9));
    assert!(near(v.mean_radius_mm, 1.0, 1e-9));
}

// ST-CAL-04
#[test]
fn st_cal_04_mean_radius_uneven() {
    let v = session_values(&shots(&[(0.0, 0.0), (6.0, 8.0), (0.0, 8.0)]));
    assert!(near(v.mean_point.x, 2.0, 1e-9) && near(v.mean_point.y, 5.333, 1e-3));
    assert!(near(v.mean_radius_mm, 4.612, 1e-3), "{}", v.mean_radius_mm);
}

// ST-CAL-05
#[test]
fn st_cal_05_score_values() {
    let v = aggregate(&[summary(50, 0, 5, 0.0, 0.0, (0.0, 0.0)), summary(90, 0, 10, 0.0, 0.0, (0.0, 0.0))]);
    assert!(near(v.score_avg, 70.0, 1e-9));
    assert_eq!((v.score_min, v.score_max), (50, 90));
    assert!(near(v.score_per_shot_avg, 140.0 / 15.0, 1e-9));
}

// ST-CAL-06
#[test]
fn st_cal_06_grouping_per_shot() {
    let v = aggregate(&[summary(0, 55, 5, 0.0, 0.0, (0.0, 0.0)), summary(0, 100, 10, 0.0, 0.0, (0.0, 0.0))]);
    assert!(near(v.grouping_score_per_shot_avg, 155.0 / 15.0, 1e-9));
}

// ST-CAL-07
#[test]
fn st_cal_07_mean_point_average() {
    let v = aggregate(&[summary(0, 0, 1, 0.0, 0.0, (1.0, 2.0)), summary(0, 0, 1, 0.0, 0.0, (3.0, -2.0))]);
    assert!(near(v.mean_point_avg.x_mm, 2.0, 1e-9) && near(v.mean_point_avg.y_mm, 0.0, 1e-9));
}

// ST-CAL-08
#[test]
fn st_cal_08_spread_and_radius_per_session() {
    let v = aggregate(&[summary(0, 0, 5, 10.0, 3.0, (0.0, 0.0)), summary(0, 0, 10, 20.0, 5.0, (0.0, 0.0))]);
    assert!(near(v.extreme_spread_mm_avg, 15.0, 1e-9) && near(v.mean_radius_mm_avg, 4.0, 1e-9));
}

// ST-CAL-09
#[test]
fn st_cal_09_counts() {
    let v = aggregate(&[summary(50, 0, 5, 0.0, 0.0, (0.0, 0.0)), summary(90, 0, 10, 0.0, 0.0, (0.0, 0.0))]);
    assert_eq!((v.session_count, v.shot_count), (2, 15));
}
