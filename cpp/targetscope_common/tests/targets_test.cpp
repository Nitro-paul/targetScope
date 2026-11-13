// doc/testing/shared_libraries.md#ring-generation-and-target-type-validation-unit
#include <gtest/gtest.h>

#include <cmath>
#include <filesystem>
#include <fstream>

#include "targetscope/targets.hpp"

using namespace targetscope;

#ifndef TARGETSCOPE_TESTDATA_DIR
#define TARGETSCOPE_TESTDATA_DIR "testdata"
#endif

static model::TargetType tt_ref() {
    model::TargetType t;
    t.name = "TT-REF";
    t.paper_width_mm = 200;
    t.paper_height_mm = 200;
    t.center_x_mm = 100;
    t.center_y_mm = 100;
    t.inner_white_radius_mm = 5;
    t.inner_white_score = 12;
    t.black_radius_mm = 30;
    t.black_score = 7;
    t.outer_radius_mm = 90;
    t.outer_score = 1;
    return t;
}

static void expect_rings(const std::vector<model::Ring>& rings, const std::vector<std::pair<int, double>>& expected) {
    ASSERT_EQ(rings.size(), expected.size());
    for (size_t i = 0; i < rings.size(); ++i) {
        EXPECT_EQ(rings[i].score, expected[i].first);
        EXPECT_NEAR(rings[i].radius_mm, expected[i].second, 1e-9);
    }
}

// LIB-TGT-01
TEST(Targets, LIB_TGT_01_RingsOfTtRef) {
    expect_rings(targets::generate_rings(tt_ref()),
                 {{12, 5}, {11, 10}, {10, 15}, {9, 20}, {8, 25}, {7, 30}, {6, 40}, {5, 50}, {4, 60}, {3, 70}, {2, 80}, {1, 90}});
}

// LIB-TGT-02
TEST(Targets, LIB_TGT_02_AnchorRadiiAreExact) {
    auto t = tt_ref();
    t.inner_white_radius_mm = 2.5;
    t.black_radius_mm = 29.75;
    t.outer_radius_mm = 77.75;
    auto rings = targets::generate_rings(t);
    EXPECT_EQ(rings[0].radius_mm, 2.5);
    EXPECT_EQ(rings[5].radius_mm, 29.75);
    EXPECT_EQ(rings[11].radius_mm, 77.75);
    for (int i = 1; i <= 5; ++i) EXPECT_NEAR(rings[i].radius_mm - rings[i - 1].radius_mm, 5.45, 1e-9);
    for (int i = 6; i <= 11; ++i) EXPECT_NEAR(rings[i].radius_mm - rings[i - 1].radius_mm, 8.0, 1e-9);
}

// LIB-TGT-03
TEST(Targets, LIB_TGT_03_NeighboringAnchorScores) {
    auto t = tt_ref();
    t.inner_white_radius_mm = 5;
    t.inner_white_score = 10;
    t.black_radius_mm = 10;
    t.black_score = 9;
    t.outer_radius_mm = 50;
    t.outer_score = 1;
    std::vector<std::pair<int, double>> expected;
    for (int s = 10; s >= 1; --s) expected.push_back({s, 5.0 * (11 - s)});
    expect_rings(targets::generate_rings(t), expected);
}

// LIB-TGT-04
TEST(Targets, LIB_TGT_04_OutermostScoreAbove1) {
    auto t = tt_ref();
    t.outer_score = 3;
    expect_rings(targets::generate_rings(t), {{12, 5}, {11, 10}, {10, 15}, {9, 20}, {8, 25}, {7, 30}, {6, 45}, {5, 60}, {4, 75}, {3, 90}});
}

// LIB-TGT-05
TEST(Targets, LIB_TGT_05_ValidTargetType) { EXPECT_FALSE(targets::validate_target_type(tt_ref()).has_value()); }

// LIB-TGT-06
TEST(Targets, LIB_TGT_06_OutermostRingTouchingThePaperEdge) {
    auto t = tt_ref();
    t.center_x_mm = 90;
    EXPECT_FALSE(targets::validate_target_type(t).has_value());
}

// LIB-TGT-07
TEST(Targets, LIB_TGT_07_InvalidTargetTypes) {
    struct Case {
        void (*change)(model::TargetType&);
        const char* rule;
    };
    const Case cases[] = {
        {[](model::TargetType& t) { t.inner_white_score = 7; }, "inner_white_score > black_score"},
        {[](model::TargetType& t) { t.black_score = 1; }, "black_score > outer_score"},
        {[](model::TargetType& t) { t.outer_score = 0; }, "outer_score >= 1"},
        {[](model::TargetType& t) { t.inner_white_radius_mm = 0; }, "inner_white_radius_mm > 0"},
        {[](model::TargetType& t) { t.inner_white_radius_mm = 30; }, "inner_white_radius_mm < black_radius_mm"},
        {[](model::TargetType& t) { t.black_radius_mm = 90; }, "black_radius_mm < outer_radius_mm"},
        {[](model::TargetType& t) { t.center_x_mm = 89; }, "outermost ring on the paper (left edge)"},
        {[](model::TargetType& t) { t.center_x_mm = 111; }, "outermost ring on the paper (right edge)"},
        {[](model::TargetType& t) { t.center_y_mm = 89; }, "outermost ring on the paper (top edge)"},
        {[](model::TargetType& t) { t.center_y_mm = 111; }, "outermost ring on the paper (bottom edge)"},
        {[](model::TargetType& t) { t.paper_width_mm = 0; }, "paper size > 0"},
    };
    for (const auto& c : cases) {
        auto t = tt_ref();
        c.change(t);
        EXPECT_EQ(targets::validate_target_type(t).value_or(""), c.rule);
    }
}

// LIB-TGT-08
TEST(Targets, LIB_TGT_08_AllExampleTargetsAllLanguages) {
    int count = 0;
    for (const auto& entry : std::filesystem::directory_iterator(std::filesystem::path(TARGETSCOPE_TESTDATA_DIR) / "target_types")) {
        if (entry.path().extension() != ".json") continue;
        std::ifstream in(entry.path());
        auto v = nlohmann::json::parse(in);
        auto t = v["target_type"].get<model::TargetType>();
        bool valid = v["valid"].get<bool>();
        EXPECT_EQ(!targets::validate_target_type(t).has_value(), valid) << entry.path();
        if (valid) {
            auto expected = v["expected_rings"].get<std::vector<model::Ring>>();
            auto rings = targets::generate_rings(t);
            ASSERT_EQ(rings.size(), expected.size());
            for (size_t i = 0; i < rings.size(); ++i) EXPECT_NEAR(rings[i].radius_mm, expected[i].radius_mm, 1e-9);
        }
        ++count;
    }
    EXPECT_GT(count, 0);
}
