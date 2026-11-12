// doc/testing/overview.md#reference-data

use targetscope_common::model::{Shot, TargetType};

pub fn tt_ref() -> TargetType {
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

pub fn shot(angle_turns: f64, radius_mm: f64) -> Shot {
    Shot { angle_turns, radius_mm }
}

pub fn s_ref() -> Vec<Shot> {
    vec![shot(0.0, 0.0), shot(0.25, 12.0), shot(0.5, 22.0), shot(0.75, 42.0), shot(0.125, 95.0)]
}

pub const DEV_REF_DIAMETER: f64 = 4.0;

pub struct Rng(pub u64);

impl Rng {
    pub fn next(&mut self) -> f64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 11) as f64 / (1u64 << 53) as f64
    }
}
