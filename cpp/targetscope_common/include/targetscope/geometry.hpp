// doc/shared_libraries.md#geometry
// doc/interfaces.md#shot
#pragma once

namespace targetscope::geometry {

struct Polar {
    double angle_turns{};
    double radius_mm{};
};

struct Point {
    double x{};
    double y{};
};

Point to_point(const Polar& p);
Polar to_polar(const Point& p);

}  // namespace targetscope::geometry
