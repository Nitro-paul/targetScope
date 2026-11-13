// doc/testing/shared_libraries.md#ring-generation-and-target-type-validation-unit

use targetscope_common::model::{Ring, TargetType};
use targetscope_common::targets::{generate_rings, validate_target_type};

fn tt_ref() -> TargetType {
    TargetType {
        id: None,
        uuid: None,
        name: "TT-REF".into(),
        paper_width_mm: 200.0,
        paper_height_mm: 200.0,
        center_x_mm: 100.0,
        center_y_mm: 100.0,
        inner_white_radius_mm: 5.0,
        inner_white_score: 12,
        black_radius_mm: 30.0,
        black_score: 7,
        outer_radius_mm: 90.0,
        outer_score: 1,
        rings: vec![],
        locked: false,
        retired: false,
    }
}

fn pairs(rings: &[Ring]) -> Vec<(i64, f64)> {
    rings.iter().map(|r| (r.score, r.radius_mm)).collect()
}

fn assert_rings(rings: &[Ring], expected: &[(i64, f64)]) {
    assert_eq!(rings.len(), expected.len(), "{rings:?}");
    for (r, (s, radius)) in rings.iter().zip(expected) {
        assert_eq!(r.score, *s);
        assert!((r.radius_mm - radius).abs() < 1e-9, "{rings:?}");
    }
}

// LIB-TGT-01
#[test]
fn lib_tgt_01_rings_of_tt_ref() {
    let expected = [(12, 5.0), (11, 10.0), (10, 15.0), (9, 20.0), (8, 25.0), (7, 30.0), (6, 40.0), (5, 50.0), (4, 60.0), (3, 70.0), (2, 80.0), (1, 90.0)];
    assert_rings(&generate_rings(&tt_ref()), &expected);
}

// LIB-TGT-02
#[test]
fn lib_tgt_02_anchor_radii_are_exact() {
    let t = TargetType { inner_white_radius_mm: 2.5, black_radius_mm: 29.75, outer_radius_mm: 77.75, ..tt_ref() };
    let rings = generate_rings(&t);
    let by_score = |s: i64| rings.iter().find(|r| r.score == s).unwrap().radius_mm;
    assert_eq!(by_score(12).to_bits(), 2.5f64.to_bits());
    assert_eq!(by_score(7).to_bits(), 29.75f64.to_bits());
    assert_eq!(by_score(1).to_bits(), 77.75f64.to_bits());
    for s in 8..=12 {
        assert!((by_score(s - 1) - by_score(s) - 5.45).abs() < 1e-9);
    }
    for s in 2..=7 {
        assert!((by_score(s - 1) - by_score(s) - 8.0).abs() < 1e-9);
    }
}

// LIB-TGT-03
#[test]
fn lib_tgt_03_neighboring_anchor_scores() {
    let t = TargetType {
        inner_white_radius_mm: 5.0,
        inner_white_score: 10,
        black_radius_mm: 10.0,
        black_score: 9,
        outer_radius_mm: 50.0,
        outer_score: 1,
        ..tt_ref()
    };
    let expected: Vec<(i64, f64)> = (1..=10).rev().map(|s| (s, 5.0 * (11 - s) as f64)).collect();
    assert_rings(&generate_rings(&t), &expected);
}

// LIB-TGT-04
#[test]
fn lib_tgt_04_outermost_score_above_1() {
    let t = TargetType { outer_score: 3, ..tt_ref() };
    let expected = [(12, 5.0), (11, 10.0), (10, 15.0), (9, 20.0), (8, 25.0), (7, 30.0), (6, 45.0), (5, 60.0), (4, 75.0), (3, 90.0)];
    assert_rings(&generate_rings(&t), &expected);
}

// LIB-TGT-05
#[test]
fn lib_tgt_05_valid_target_type() {
    assert_eq!(validate_target_type(&tt_ref()), Ok(()));
}

// LIB-TGT-06
#[test]
fn lib_tgt_06_outermost_ring_touching_the_paper_edge() {
    assert_eq!(validate_target_type(&TargetType { center_x_mm: 90.0, ..tt_ref() }), Ok(()));
}

// LIB-TGT-07
#[test]
fn lib_tgt_07_invalid_target_types() {
    let cases: Vec<(TargetType, &str)> = vec![
        (TargetType { inner_white_score: 7, ..tt_ref() }, "inner_white_score > black_score"),
        (TargetType { black_score: 1, ..tt_ref() }, "black_score > outer_score"),
        (TargetType { outer_score: 0, ..tt_ref() }, "outer_score >= 1"),
        (TargetType { inner_white_radius_mm: 0.0, ..tt_ref() }, "inner_white_radius_mm > 0"),
        (TargetType { inner_white_radius_mm: 30.0, ..tt_ref() }, "inner_white_radius_mm < black_radius_mm"),
        (TargetType { black_radius_mm: 90.0, ..tt_ref() }, "black_radius_mm < outer_radius_mm"),
        (TargetType { center_x_mm: 89.0, ..tt_ref() }, "outermost ring on the paper (left edge)"),
        (TargetType { center_x_mm: 111.0, ..tt_ref() }, "outermost ring on the paper (right edge)"),
        (TargetType { center_y_mm: 89.0, ..tt_ref() }, "outermost ring on the paper (top edge)"),
        (TargetType { center_y_mm: 111.0, ..tt_ref() }, "outermost ring on the paper (bottom edge)"),
        (TargetType { paper_width_mm: 0.0, ..tt_ref() }, "paper size > 0"),
    ];
    for (t, rule) in cases {
        assert_eq!(validate_target_type(&t), Err(rule.to_string()));
    }
}

// LIB-TGT-08
#[test]
fn lib_tgt_08_all_example_targets_all_languages() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/target_types");
    let mut count = 0;
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let t: TargetType = serde_json::from_value(v["target_type"].clone()).unwrap();
        let valid = v["valid"].as_bool().unwrap();
        assert_eq!(validate_target_type(&t).is_ok(), valid, "{path:?}");
        if valid {
            let expected: Vec<Ring> = serde_json::from_value(v["expected_rings"].clone()).unwrap();
            assert_rings(&generate_rings(&t), &pairs(&expected));
        }
        count += 1;
    }
    assert!(count > 0);
}
