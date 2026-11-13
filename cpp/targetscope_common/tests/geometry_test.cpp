// doc/testing/shared_libraries.md#geometry-unit
#include <gtest/gtest.h>

#include <cmath>
#include <random>

#include "targetscope/geometry.hpp"

using namespace targetscope::geometry;

// LIB-GEO-01
TEST(Geometry, LIB_GEO_01_PolarToCartesianFourDirections) {
    const double in[4][2] = {{0, 10}, {0.25, 10}, {0.5, 10}, {0.75, 10}};
    const double out[4][2] = {{0, 10}, {10, 0}, {0, -10}, {-10, 0}};
    for (int i = 0; i < 4; ++i) {
        auto p = to_point({in[i][0], in[i][1]});
        EXPECT_NEAR(p.x, out[i][0], 1e-9);
        EXPECT_NEAR(p.y, out[i][1], 1e-9);
    }
}

// LIB-GEO-02
TEST(Geometry, LIB_GEO_02_PolarToCartesianDiagonal) {
    auto p = to_point({0.125, 10});
    EXPECT_NEAR(p.x, 7.0710678, 1e-7);
    EXPECT_NEAR(p.y, 7.0710678, 1e-7);
}

// LIB-GEO-03
TEST(Geometry, LIB_GEO_03_CartesianToPolarCenter) {
    auto p = to_polar({0, 0});
    EXPECT_EQ(p.angle_turns, 0.0);
    EXPECT_EQ(p.radius_mm, 0.0);
}

// LIB-GEO-04
TEST(Geometry, LIB_GEO_04_CartesianToPolarFourDirections) {
    const double in[4][2] = {{0, 10}, {10, 0}, {0, -10}, {-10, 0}};
    const double out[4] = {0, 0.25, 0.5, 0.75};
    for (int i = 0; i < 4; ++i) {
        auto p = to_polar({in[i][0], in[i][1]});
        EXPECT_NEAR(p.angle_turns, out[i], 1e-12);
        EXPECT_NEAR(p.radius_mm, 10, 1e-12);
    }
}

// LIB-GEO-05
TEST(Geometry, LIB_GEO_05_AngleStaysBelow1) { EXPECT_EQ(to_polar({-1e-17, 10}).angle_turns, 0.0); }

// LIB-GEO-06
TEST(Geometry, LIB_GEO_06_RoundTrip) {
    std::mt19937_64 rng(7);
    std::uniform_real_distribution<double> a(0.0, 1.0), r(0.001, 200.0);
    for (int i = 0; i < 100000; ++i) {
        Polar in{a(rng), r(rng)};
        auto p1 = to_point(in);
        auto back = to_polar(p1);
        auto p2 = to_point(back);
        ASSERT_NEAR(p1.x, p2.x, 1e-9);
        ASSERT_NEAR(p1.y, p2.y, 1e-9);
        ASSERT_LT(back.angle_turns, 1.0);
    }
}

// LIB-GEO-07
TEST(Geometry, LIB_GEO_07_RustAndCppAgree) { GTEST_SKIP() << "doc/testing/shared_libraries.md#geometry-unit LIB-GEO-07"; }
