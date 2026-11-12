// doc/testing/analyzer.md#session-analysis-unit

mod common;

use analyzer::scoring::{analyze, total_score};
use common::{s_ref, shot, tt_ref, Rng, DEV_REF_DIAMETER};
use proptest::prelude::*;
use targetscope_common::geometry::Point;
use targetscope_common::model::{OffsetMm, Ring, Shot};
use targetscope_common::targets::generate_rings;

fn near(a: f64, b: f64, tol: f64) -> bool {
    (a - b).abs() <= tol
}

// AN-ANL-01
#[test]
fn an_anl_01_no_shots() {
    let r = analyze(&[], &tt_ref(), DEV_REF_DIAMETER);
    assert!(r.shot_scores.is_empty());
    assert_eq!((r.score, r.grouping_score), (0, 0));
    assert_eq!(r.grouping_shift, OffsetMm { x_mm: 0.0, y_mm: 0.0 });
    assert_eq!(r.mean_point, None);
}

// AN-ANL-02
#[test]
fn an_anl_02_s_ref() {
    let r = analyze(&s_ref(), &tt_ref(), DEV_REF_DIAMETER);
    assert_eq!(r.shot_scores, vec![12, 11, 9, 6, 0]);
    assert_eq!(r.score, 38);
    let m = r.mean_point.unwrap();
    assert!(near(m.x_mm, 7.435, 1e-3) && near(m.y_mm, 9.035, 1e-3), "{m:?}");
}

// AN-ANL-03
#[test]
fn an_anl_03_one_shot_off_center() {
    let r = analyze(&[shot(0.25, 50.0)], &tt_ref(), DEV_REF_DIAMETER);
    assert_eq!((r.score, r.grouping_score), (5, 12));
    assert!(near(r.grouping_shift.x_mm, 43.0, 0.05) && near(r.grouping_shift.y_mm, 0.0, 0.05), "{:?}", r.grouping_shift);
}

// AN-ANL-04
#[test]
fn an_anl_04_three_shots_on_one_spot() {
    let r = analyze(&[shot(0.25, 50.0); 3], &tt_ref(), DEV_REF_DIAMETER);
    assert_eq!((r.score, r.grouping_score), (15, 36));
    assert!(near(r.grouping_shift.x_mm, 43.0, 0.05) && near(r.grouping_shift.y_mm, 0.0, 0.05), "{:?}", r.grouping_shift);
}

// AN-ANL-05
#[test]
fn an_anl_05_tie_no_shift_is_better() {
    let r = analyze(&[shot(0.25, 10.0), shot(0.75, 10.0)], &tt_ref(), DEV_REF_DIAMETER);
    assert_eq!(r.shot_scores, vec![11, 11]);
    assert_eq!((r.score, r.grouping_score), (22, 22));
    assert_eq!(r.grouping_shift, OffsetMm { x_mm: 0.0, y_mm: 0.0 });
}

// AN-ANL-06
#[test]
fn an_anl_06_mean_point() {
    let m = analyze(&[shot(0.0, 10.0), shot(0.25, 10.0)], &tt_ref(), DEV_REF_DIAMETER).mean_point.unwrap();
    assert!(near(m.x_mm, 5.0, 1e-9) && near(m.y_mm, 5.0, 1e-9));
}

fn shots_strategy() -> impl Strategy<Value = Vec<Shot>> {
    prop::collection::vec((0.0f64..1.0, 0.0f64..100.0).prop_map(|(a, r)| shot(a, r)), 1..=60)
}

// AN-ANL-07
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]
    #[test]
    #[ignore = "long"]
    fn an_anl_07_grouping_never_worse(shots in shots_strategy()) {
        let r = analyze(&shots, &tt_ref(), DEV_REF_DIAMETER);
        prop_assert!(r.grouping_score >= r.score);
    }
}

// AN-ANL-07
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    #[test]
    fn an_anl_07_grouping_never_worse_quick(shots in shots_strategy()) {
        let r = analyze(&shots, &tt_ref(), DEV_REF_DIAMETER);
        prop_assert!(r.grouping_score >= r.score);
    }
}

// AN-ANL-08
#[test]
fn an_anl_08_same_result_every_time() {
    let first = serde_json::to_string(&analyze(&s_ref(), &tt_ref(), DEV_REF_DIAMETER)).unwrap();
    for _ in 0..100 {
        assert_eq!(serde_json::to_string(&analyze(&s_ref(), &tt_ref(), DEV_REF_DIAMETER)).unwrap(), first);
    }
}

// AN-ANL-09
#[test]
fn an_anl_09_rings_in_the_request_are_ignored() {
    let mut t = tt_ref();
    t.rings = vec![Ring { score: 99, radius_mm: 1.0 }, Ring { score: 50, radius_mm: 2.0 }, Ring { score: 1, radius_mm: 3.0 }];
    assert_eq!(analyze(&s_ref(), &t, DEV_REF_DIAMETER), analyze(&s_ref(), &tt_ref(), DEV_REF_DIAMETER));
}

// doc/testing/analyzer.md#session-analysis-unit
fn brute_force(points: &[Point], rings: &[Ring], b: f64) -> i64 {
    let step = 0.01;
    let reach = rings.last().unwrap().radius_mm + b;
    let min_x = points.iter().map(|p| p.x).fold(f64::INFINITY, f64::min) - reach;
    let max_x = points.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max) + reach;
    let min_y = points.iter().map(|p| p.y).fold(f64::INFINITY, f64::min) - reach;
    let max_y = points.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max) + reach;
    let (nx, ny) = (((max_x - min_x) / step).ceil() as i64, ((max_y - min_y) / step).ceil() as i64);
    let at = |i: i64, j: i64| Point { x: min_x + i as f64 * step, y: min_y + j as f64 * step };
    let bound = |i0: i64, i1: i64, j0: i64, j1: i64| -> i64 {
        let (a, c) = (at(i0, j0), at(i1, j1));
        points
            .iter()
            .map(|p| {
                let dx = (a.x - p.x).max(0.0).max(p.x - c.x);
                let dy = (a.y - p.y).max(0.0).max(p.y - c.y);
                analyzer::scoring::shot_score(dx.hypot(dy), rings, b)
            })
            .sum()
    };
    let mut best = total_score(points, Point { x: 0.0, y: 0.0 }, rings, b);
    let mut stack = vec![(0, nx, 0, ny)];
    while let Some((i0, i1, j0, j1)) = stack.pop() {
        if bound(i0, i1, j0, j1) <= best {
            continue;
        }
        if i1 - i0 <= 1 && j1 - j0 <= 1 {
            for i in i0..=i1 {
                for j in j0..=j1 {
                    best = best.max(total_score(points, at(i, j), rings, b));
                }
            }
            continue;
        }
        let (im, jm) = ((i0 + i1) / 2, (j0 + j1) / 2);
        if i1 - i0 >= j1 - j0 {
            stack.push((i0, im, j0, j1));
            stack.push((im + 1, i1, j0, j1));
        } else {
            stack.push((i0, i1, j0, jm));
            stack.push((i0, i1, jm + 1, j1));
        }
    }
    best
}

// AN-ANL-10
#[test]
fn an_anl_10_compared_with_brute_force() {
    let rings = generate_rings(&tt_ref());
    let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
    let (mut equal, total) = (0, 1000);
    for _ in 0..total {
        let n = 2 + (rng.next() * 9.0) as usize;
        let shots: Vec<Shot> = (0..n).map(|_| shot(rng.next(), rng.next() * 40.0)).collect();
        let points: Vec<Point> = shots.iter().map(|s| Point::from(*s)).collect();
        let found = analyze(&shots, &tt_ref(), DEV_REF_DIAMETER).grouping_score;
        let reference = brute_force(&points, &rings, DEV_REF_DIAMETER / 2.0);
        if found == reference {
            equal += 1;
        }
    }
    let rate = equal as f64 / total as f64;
    println!("AN-ANL-10 miss rate: {:.1} %", (1.0 - rate) * 100.0);
    assert!(rate >= 0.99, "{rate}");
}
