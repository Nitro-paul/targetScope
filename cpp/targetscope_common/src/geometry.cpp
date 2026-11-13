// doc/shared_libraries.md#geometry
#include "targetscope/geometry.hpp"

#include <cmath>
#include <numbers>

namespace targetscope::geometry {

Point to_point(const Polar& p) {
    const double a = 2.0 * std::numbers::pi * p.angle_turns;
    return {p.radius_mm * std::sin(a), p.radius_mm * std::cos(a)};
}

Polar to_polar(const Point& p) {
    const double r = std::hypot(p.x, p.y);
    if (r == 0.0) return {0.0, 0.0};
    double a = std::atan2(p.x, p.y) / (2.0 * std::numbers::pi);
    if (a < 0.0) a += 1.0;
    if (a >= 1.0) a = 0.0;
    return {a, r};
}

}
