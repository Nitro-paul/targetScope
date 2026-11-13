// doc/shared_libraries.md#geometry
// doc/interfaces.md#shot

use std::f64::consts::TAU;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Polar {
    pub angle_turns: f64,
    pub radius_mm: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn distance(&self, other: &Point) -> f64 {
        (self.x - other.x).hypot(self.y - other.y)
    }

    pub fn length(&self) -> f64 {
        self.x.hypot(self.y)
    }
}

impl From<Polar> for Point {
    fn from(p: Polar) -> Self {
        let a = TAU * p.angle_turns;
        Point { x: p.radius_mm * a.sin(), y: p.radius_mm * a.cos() }
    }
}

impl From<Point> for Polar {
    fn from(p: Point) -> Self {
        let radius_mm = p.x.hypot(p.y);
        if radius_mm == 0.0 {
            return Polar { angle_turns: 0.0, radius_mm };
        }
        let mut angle_turns = p.x.atan2(p.y) / TAU;
        if angle_turns < 0.0 {
            angle_turns += 1.0;
        }
        if angle_turns >= 1.0 {
            angle_turns = 0.0;
        }
        Polar { angle_turns, radius_mm }
    }
}

impl From<crate::model::Shot> for Point {
    fn from(s: crate::model::Shot) -> Self {
        Polar { angle_turns: s.angle_turns, radius_mm: s.radius_mm }.into()
    }
}

impl From<Point> for crate::model::Shot {
    fn from(p: Point) -> Self {
        let polar = Polar::from(p);
        crate::model::Shot { angle_turns: polar.angle_turns, radius_mm: polar.radius_mm }
    }
}
