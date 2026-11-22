#include <gtest/gtest.h>

#include <algorithm>

#include "hole_detector.hpp"
#include "rig.hpp"

// IP-HOL
// doc/testing/image_processor.md#hole-detection-integration

using namespace image_processor;
using namespace image_processor::testing;
using targetscope::ErrorCode;
using targetscope::ServiceError;

namespace {

const std::vector<cv::Point2d> FIVE{{80, 100}, {120, 105}, {100, 70}, {60, 60}, {140, 140}};

Scene base(std::vector<cv::Point2d> holes = FIVE) {
    Scene s;
    s.holes_mm = std::move(holes);
    return s;
}

Scene plus(Scene s, cv::Point2d hole) {
    s.holes_mm.push_back(hole);
    return s;
}

}

// IP-HOL-01
TEST(HoleDetection, IP_HOL_01_HolesAlreadyThere) {
    Rig r;
    Scene s = base();
    r.pipeline.start(s.target, s.bullet_mm);
    r.feed(render(s), 8);
    ASSERT_EQ(r.state.live->holes.size(), 5u);
    for (auto& h : r.state.live->holes) EXPECT_FALSE(h.counted);
    EXPECT_TRUE(r.events_of("shot_detected").empty());
    for (auto& truth : s.holes_mm) {
        double best = 1e9;
        for (auto& h : r.state.live->holes) best = std::min(best, dist_to_truth(h.paper_mm, truth));
        EXPECT_LT(best, 1.0);
    }
}

// IP-HOL-02
TEST(HoleDetection, IP_HOL_02_CountingOff) {
    Rig r;
    Scene s = base();
    r.start(s);
    r.feed(render(plus(s, {100, 120})), 10);
    ASSERT_EQ(r.state.live->holes.size(), 6u);
    EXPECT_FALSE(r.state.live->holes.back().counted);
    EXPECT_TRUE(r.events_of("shot_detected").empty());
}

// IP-HOL-03
TEST(HoleDetection, IP_HOL_03_CountingWithoutCalibration) {
    Rig r;
    r.pipeline.start(tt_ref(), 4.0);
    try {
        r.pipeline.set_counting(true);
        FAIL();
    } catch (const ServiceError& e) {
        EXPECT_EQ(e.code, ErrorCode::Unavailable);
    }
}

// IP-HOL-04
TEST(HoleDetection, IP_HOL_04_NewShot) {
    Rig r;
    Scene s = base();
    r.start(s, true);
    cv::Mat with_new = render(plus(s, {100, 120}));
    r.feed(with_new, 4);
    EXPECT_TRUE(r.events_of("shot_detected").empty());
    r.feed(with_new, 1);
    auto shots = r.events_of("shot_detected");
    ASSERT_EQ(shots.size(), 1u);
    r.feed(with_new, 10);
    EXPECT_EQ(r.events_of("shot_detected").size(), 1u);
    const auto& e = shots[0];
    EXPECT_EQ(e["live_paper"], 1);
    EXPECT_GT(e["confidence"].get<double>(), 0.5);
    auto truth = paper_to_shot({100, 120}, s.target);
    auto t = r.state.live->target_type;
    cv::Point2d got = shot_to_paper({e["angle_turns"].get<double>(), e["radius_mm"].get<double>()}, t);
    EXPECT_LT(dist_to_truth(got, {100, 120}), 1.0);
    EXPECT_NEAR(e["radius_mm"].get<double>(), truth.radius_mm, 1.0);
    ASSERT_TRUE(r.state.live->holes.back().counted);
}

// IP-HOL-05
TEST(HoleDetection, IP_HOL_05_ShortArtifact) {
    Rig r;
    Scene s = base();
    r.start(s, true);
    r.feed(render(plus(s, {100, 120})), 3);
    ASSERT_EQ(r.state.live->candidates.size(), 1u);
    r.feed(render(s), 3);
    EXPECT_TRUE(r.state.live->candidates.empty());
    r.feed(render(s), 5);
    EXPECT_TRUE(r.events_of("shot_detected").empty());
    EXPECT_EQ(r.state.live->holes.size(), 5u);
}

// IP-HOL-06
TEST(HoleDetection, IP_HOL_06_ConfirmationSetting) {
    Rig r;
    r.state.runtime.confirm_frames = 10;
    Scene s = base();
    r.start(s, true);
    cv::Mat with_new = render(plus(s, {100, 120}));
    r.feed(with_new, 9);
    EXPECT_TRUE(r.events_of("shot_detected").empty());
    r.feed(with_new, 1);
    EXPECT_EQ(r.events_of("shot_detected").size(), 1u);
}

// IP-HOL-07
TEST(HoleDetection, IP_HOL_07_HandInFrontOfThePaper) {
    Rig r;
    Scene s = base();
    r.start(s, true);
    Scene hand = plus(s, {100, 150});
    hand.hand_fraction = 0.20;
    r.feed(render(hand), 20);
    EXPECT_TRUE(r.events_of("shot_detected").empty());
    EXPECT_EQ(r.state.live->holes.size(), 5u);
    r.feed(render(plus(s, {100, 150})), 8);
    EXPECT_EQ(r.events_of("shot_detected").size(), 1u);
}

// IP-HOL-08
TEST(HoleDetection, IP_HOL_08_Shadow) {
    Rig r;
    Scene s = base();
    r.start(s, true);
    Scene shadow = s;
    shadow.shadow_fraction = 0.06;
    r.feed(render(shadow), 20);
    EXPECT_TRUE(r.events_of("shot_detected").empty());
    EXPECT_EQ(r.state.live->holes.size(), 5u);
}

// IP-HOL-09
TEST(HoleDetection, IP_HOL_09_SizeFilter) {
    for (double mm : {2.0, 10.0}) {
        Rig r;
        Scene s = base();
        r.start(s, true);
        Scene spot = s;
        spot.extra_spot_mm = mm;
        spot.extra_spot_pos = {160, 60};
        r.feed(render(spot), 12);
        EXPECT_TRUE(r.events_of("shot_detected").empty()) << mm;
    }
}

// IP-HOL-10
TEST(HoleDetection, IP_HOL_10_ShapeFilter) {
    Rig r;
    Scene s = base();
    r.start(s, true);
    Scene line = s;
    line.extra_line = true;
    r.feed(render(line), 12);
    EXPECT_TRUE(r.events_of("shot_detected").empty());
}

// IP-HOL-11
TEST(HoleDetection, IP_HOL_11_OverlappingHoles) {
    Rig r;
    Scene s = base({{80, 100}, {130, 100}});
    r.start(s, true);
    r.feed(render(plus(s, {82.4, 100})), 12);
    auto shots = r.events_of("shot_detected");
    ASSERT_EQ(shots.size(), 1u);
    EXPECT_LT(shots[0]["confidence"].get<double>(), 0.5);
    auto t = r.state.live->target_type;
    cv::Point2d got = shot_to_paper({shots[0]["angle_turns"].get<double>(), shots[0]["radius_mm"].get<double>()}, t);
    EXPECT_LT(dist_to_truth(got, {82.4, 100}), 1.5);
}

// IP-HOL-12
TEST(HoleDetection, IP_HOL_12_HolesInTheBlackAndInTheWhite) {
    Rig r;
    Scene s = base({{80, 100}});
    r.start(s, true);
    Scene two = plus(plus(s, {100, 85}), {150, 50});
    r.feed(render(two), 10);
    EXPECT_EQ(r.events_of("shot_detected").size(), 2u);
}

// IP-HOL-13
TEST(HoleDetection, IP_HOL_13_Uncount) {
    Rig r;
    Scene s = base({});
    r.start(s, true);
    Scene one = plus(s, {90, 100});
    Scene two = plus(one, {110, 100});
    Scene three = plus(two, {100, 120});
    r.feed(render(one), 7);
    r.feed(render(two), 7);
    r.feed(render(three), 7);
    auto shots = r.events_of("shot_detected");
    ASSERT_EQ(shots.size(), 3u);
    r.pipeline.uncount_hole(shots[1]["hole_id"].get<int>());
    int counted = 0;
    for (auto& h : r.state.live->holes) counted += h.counted;
    EXPECT_EQ(counted, 2);
    cv::Mat out = *r.pipeline.process(render(three), 99);
    EXPECT_EQ(out.cols, 1200);
}

// IP-HOL-14
TEST(HoleDetection, IP_HOL_14_UncountAnUnknownHole) {
    Rig r;
    Scene s = base();
    r.start(s, true);
    const auto before = r.state.live->holes.size();
    r.pipeline.uncount_hole(9999);
    EXPECT_EQ(r.state.live->holes.size(), before);
}

// IP-HOL-15
TEST(HoleDetection, IP_HOL_15_HoleIds) {
    Rig r;
    Scene s = base({});
    r.start(s, true);
    for (int i = 0; i < 10; ++i) {
        s.holes_mm.push_back({70.0 + 6 * (i % 5), i < 5 ? 90.0 : 115.0});
        r.feed(render(s), 7);
    }
    auto shots = r.events_of("shot_detected");
    ASSERT_EQ(shots.size(), 10u);
    for (size_t i = 1; i < shots.size(); ++i)
        EXPECT_GT(shots[i]["hole_id"].get<int>(), shots[i - 1]["hole_id"].get<int>());
}

TEST(HoleDetection, Candidates_Confirmation) {
    std::vector<Candidate> c;
    Blob b;
    b.paper_mm = {10, 10};
    b.diameter_mm = 4;
    b.roundness = 0.9;
    for (int i = 0; i < 4; ++i) EXPECT_TRUE(update_candidates(c, {b}, 4.0, 5).empty());
    EXPECT_EQ(update_candidates(c, {b}, 4.0, 5).size(), 1u);
    EXPECT_TRUE(c.empty());
}
