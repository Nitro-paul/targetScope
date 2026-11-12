//! Polar <-> Cartesian conversion (doc/shared_libraries.md#geometry).

/// Same fields as `Shot`: angle clockwise from the top in turns, radius in mm.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Polar {
    pub angle_turns: f64,
    pub radius_mm: f64,
}

/// mm, origin at the ring center, x right, y up.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl From<Polar> for Point {
    /// x = radius_mm·sin(2π·angle_turns), y = radius_mm·cos(2π·angle_turns)
    fn from(p: Polar) -> Self {
        let _ = p;
        todo!()
    }
}

impl From<Point> for Polar {
    /// radius_mm = √(x²+y²), angle_turns = atan2(x, y) / 2π, normalized to [0, 1); radius 0 → angle 0
    fn from(p: Point) -> Self {
        let _ = p;
        todo!()
    }
}
