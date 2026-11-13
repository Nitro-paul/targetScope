// doc/testing/analyzer.md#shot-score-unit

mod common;

use analyzer::scoring::shot_score;
use targetscope_common::targets::generate_rings;

fn score(d: f64) -> i64 {
    shot_score(d, &generate_rings(&common::tt_ref()), 2.0)
}

// AN-SCR-01
#[test]
fn an_scr_01_center() {
    assert_eq!(score(0.0), 12);
}

// AN-SCR-02
#[test]
fn an_scr_02_inside_the_white_ring() {
    assert_eq!(score(6.9), 12);
}

// AN-SCR-03
#[test]
fn an_scr_03_exactly_on_the_line() {
    assert_eq!(score(7.0), 12);
}

// AN-SCR-04
#[test]
fn an_scr_04_within_the_tolerance() {
    assert_eq!(score(7.0 + 5e-10), 12);
}

// AN-SCR-05
#[test]
fn an_scr_05_just_outside() {
    assert_eq!(score(7.001), 11);
}

// AN-SCR-06
#[test]
fn an_scr_06_rings_11_to_7_on_their_lines() {
    for (d, s) in [(12.0, 11), (17.0, 10), (22.0, 9), (27.0, 8), (32.0, 7)] {
        assert_eq!(score(d), s, "{d}");
    }
}

// AN-SCR-07
#[test]
fn an_scr_07_just_outside_the_black() {
    assert_eq!(score(32.001), 6);
}

// AN-SCR-08
#[test]
fn an_scr_08_outermost_ring_on_its_line() {
    assert_eq!(score(92.0), 1);
}

// AN-SCR-09
#[test]
fn an_scr_09_miss() {
    assert_eq!(score(92.001), 0);
    assert_eq!(score(500.0), 0);
}

// AN-SCR-10
#[test]
fn an_scr_10_without_bullet_radius() {
    let rings = generate_rings(&common::tt_ref());
    assert_eq!(shot_score(5.0, &rings, 0.0), 12);
    assert_eq!(shot_score(5.001, &rings, 0.0), 11);
}
