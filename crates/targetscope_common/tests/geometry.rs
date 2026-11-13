// doc/testing/shared_libraries.md#geometry-unit

use targetscope_common::geometry::{Point, Polar};

fn to_point(a: f64, r: f64) -> Point {
    Polar { angle_turns: a, radius_mm: r }.into()
}

fn to_polar(x: f64, y: f64) -> Polar {
    Point { x, y }.into()
}

fn close(a: f64, b: f64, tol: f64) -> bool {
    (a - b).abs() <= tol
}

// LIB-GEO-01
#[test]
fn lib_geo_01_polar_to_cartesian_four_directions() {
    for ((a, r), (x, y)) in
        [((0.0, 10.0), (0.0, 10.0)), ((0.25, 10.0), (10.0, 0.0)), ((0.5, 10.0), (0.0, -10.0)), ((0.75, 10.0), (-10.0, 0.0))]
    {
        let p = to_point(a, r);
        assert!(close(p.x, x, 1e-9) && close(p.y, y, 1e-9), "{a} {r} -> {p:?}");
    }
}

// LIB-GEO-02
#[test]
fn lib_geo_02_polar_to_cartesian_diagonal() {
    let p = to_point(0.125, 10.0);
    assert!(close(p.x, 7.0710678, 1e-7) && close(p.y, 7.0710678, 1e-7));
}

// LIB-GEO-03
#[test]
fn lib_geo_03_cartesian_to_polar_center() {
    assert_eq!(to_polar(0.0, 0.0), Polar { angle_turns: 0.0, radius_mm: 0.0 });
}

// LIB-GEO-04
#[test]
fn lib_geo_04_cartesian_to_polar_four_directions() {
    for ((x, y), (a, r)) in
        [((0.0, 10.0), (0.0, 10.0)), ((10.0, 0.0), (0.25, 10.0)), ((0.0, -10.0), (0.5, 10.0)), ((-10.0, 0.0), (0.75, 10.0))]
    {
        let p = to_polar(x, y);
        assert!(close(p.angle_turns, a, 1e-12) && close(p.radius_mm, r, 1e-12), "{x} {y} -> {p:?}");
    }
}

// LIB-GEO-05
#[test]
fn lib_geo_05_angle_stays_below_1() {
    let p = to_polar(-1e-17, 10.0);
    assert_eq!(p.angle_turns, 0.0);
}

// LIB-GEO-06
#[test]
fn lib_geo_06_round_trip() {
    let mut seed: u64 = 0x2545_f491_4f6c_dd1d;
    let mut next = || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        (seed >> 11) as f64 / (1u64 << 53) as f64
    };
    for _ in 0..100_000 {
        let a = next();
        let r = 0.001 + next() * (200.0 - 0.001);
        let back: Polar = to_point(a, r).into();
        let p1 = to_point(a, r);
        let p2 = to_point(back.angle_turns, back.radius_mm);
        assert!(close(p1.x, p2.x, 1e-9) && close(p1.y, p2.y, 1e-9), "{a} {r} -> {back:?}");
        assert!(back.angle_turns < 1.0);
    }
}
