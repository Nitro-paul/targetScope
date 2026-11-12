// Polar <-> Cartesian conversion (doc/shared_libraries.md#geometry), same as in Rust.
#pragma once

namespace targetscope::geometry {

// angle clockwise from the top in turns, radius in mm
struct Polar {
    double angle_turns{};
    double radius_mm{};
};

// mm, origin at the ring center, x right, y up
struct Point {
    double x{};
    double y{};
};

// x = radius_mm·sin(2π·angle_turns), y = radius_mm·cos(2π·angle_turns)
Point to_point(const Polar& p);
// radius_mm = √(x²+y²), angle_turns = atan2(x, y) / 2π, normalized to [0, 1); radius 0 → angle 0
Polar to_polar(const Point& p);

}  // namespace targetscope::geometry
