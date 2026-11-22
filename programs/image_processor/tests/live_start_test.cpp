#include <gtest/gtest.h>

#include "rig.hpp"

// IP-LIV
// doc/testing/image_processor.md#live-mode-start-and-calibration-integration

using namespace image_processor;
using namespace image_processor::testing;
using targetscope::ErrorCode;
using targetscope::ServiceError;

namespace {

Scene with_holes(std::vector<cv::Point2d> holes) {
    Scene s;
    s.holes_mm = std::move(holes);
    return s;
}

}

// IP-LIV-01
TEST(LiveStart, IP_LIV_01_StartWithoutFrames) {
    Rig r;
    auto status = r.pipeline.start(tt_ref(), 4.0);
    EXPECT_EQ(status.state, "not_calibrated");
    EXPECT_EQ(status.message, "no camera frames");
    EXPECT_FALSE(r.last_out);
}

// IP-LIV-02
TEST(LiveStart, IP_LIV_02_AutomaticCalibration) {
    Rig r;
    Scene s;
    r.pipeline.start(s.target, s.bullet_mm);
    r.feed(render(s));
    auto status = r.pipeline.status();
    EXPECT_EQ(status.state, "calibrated");
    EXPECT_EQ(status.method, "auto");
    ASSERT_TRUE(status.corners);
    ASSERT_TRUE(r.last_out);
    EXPECT_EQ(r.last_out->cols, 1200);
    EXPECT_EQ(r.last_out->rows, 1200);
    auto changed = r.events_of("calibration_changed");
    ASSERT_FALSE(changed.empty());
    EXPECT_EQ(changed.back()["state"], "calibrated");
    EXPECT_EQ(changed.back()["method"], "auto");
}

// IP-LIV-03
TEST(LiveStart, IP_LIV_03_StartAgain) {
    Rig r;
    Scene s = with_holes({{80, 100}, {120, 100}});
    r.pipeline.start(s.target, s.bullet_mm);
    r.feed(render(s), 2);
    ASSERT_EQ(r.state.live->holes.size(), 2u);
    r.state.live->live_paper = 3;
    r.pipeline.start(s.target, s.bullet_mm);
    EXPECT_EQ(r.state.live->live_paper, 1);
    EXPECT_TRUE(r.state.live->holes.empty());
    r.feed(render(s), 2);
    EXPECT_EQ(r.pipeline.status().state, "calibrated");
    EXPECT_EQ(r.state.live->holes.size(), 2u);
}

// IP-LIV-04
TEST(LiveStart, IP_LIV_04_ManualCalibrationOutsideLiveMode) {
    Rig r;
    Scene s;
    try {
        r.pipeline.calibrate(to_corners(project_corners(s)));
        FAIL();
    } catch (const ServiceError& e) {
        EXPECT_EQ(e.code, ErrorCode::InvalidState);
    }
}

// IP-LIV-05
TEST(LiveStart, IP_LIV_05_ManualCalibration) {
    Rig r;
    Scene s;
    s.pose.yaw_deg = 20;
    auto corners = to_corners(project_corners(s));
    r.pipeline.start(s.target, s.bullet_mm);
    r.state.newest_raw = render(s);
    r.state.last_frame_time = std::chrono::steady_clock::now();
    auto status = r.pipeline.calibrate(corners);
    EXPECT_EQ(status.state, "calibrated");
    EXPECT_EQ(status.method, "manual");
    ASSERT_TRUE(status.corners);
    EXPECT_EQ(status.corners->top_left.x, corners.top_left.x);
    EXPECT_EQ(status.corners->bottom_right.y, corners.bottom_right.y);
}

// IP-LIV-06
TEST(LiveStart, IP_LIV_06_ManualCalibrationWithLensCorrection) {
    Rig r;
    Scene s;
    s.holes_mm = {{60, 70}, {150, 130}};
    s.distortion_k1 = -0.12;
    r.install_lens(-0.12, s.image);
    auto raw_corners = r.state.corrector->to_raw(project_corners(s));
    r.pipeline.start(s.target, s.bullet_mm);
    r.state.newest_raw = render(s);
    r.state.last_frame_time = std::chrono::steady_clock::now();
    auto status = r.pipeline.calibrate(to_corners(raw_corners));
    ASSERT_TRUE(status.corners);
    EXPECT_NEAR(status.corners->top_left.x, raw_corners[0].x, 1e-3);
    ASSERT_EQ(r.state.live->holes.size(), 2u);
    for (auto& h : r.state.live->holes) {
        double best = 1e9;
        for (auto& truth : s.holes_mm) best = std::min(best, dist_to_truth(h.paper_mm, truth));
        EXPECT_LT(best, 0.5);
    }
}

// IP-LIV-07
TEST(LiveStart, IP_LIV_07_StatusOutsideLiveMode) {
    Rig r;
    auto status = r.pipeline.status();
    EXPECT_EQ(status.state, "not_calibrated");
    EXPECT_EQ(status.message, "live mode not active");
}

namespace {

void paper_moves(Rig& r, double shift_mm, bool manual) {
    Scene s = with_holes({{80, 100}, {120, 100}, {100, 70}});
    r.pipeline.start(s.target, s.bullet_mm);
    if (manual) {
        r.state.newest_raw = render(s);
        r.state.last_frame_time = std::chrono::steady_clock::now();
        r.pipeline.calibrate(to_corners(project_corners(s)));
    }
    r.feed(render(s), 31);
    r.pipeline.set_counting(true);
    Scene moved = s;
    moved.pose.dx_mm = shift_mm;
    r.feed(render(moved), 65);
}

}

// IP-LIV-08
TEST(LiveStart, IP_LIV_08_PaperMovesAutomatic) {
    Rig r;
    paper_moves(r, 5, false);
    EXPECT_EQ(r.pipeline.status().state, "calibrated");
    EXPECT_GE(r.events_of("calibration_changed").size(), 2u);
    ASSERT_EQ(r.state.live->holes.size(), 3u);
    EXPECT_TRUE(r.events_of("shot_detected").empty());
    for (auto& h : r.state.live->holes) EXPECT_FALSE(h.counted);
}

// IP-LIV-09
TEST(LiveStart, IP_LIV_09_PaperMovesManual) {
    Rig r;
    paper_moves(r, 5, true);
    EXPECT_EQ(r.pipeline.status().state, "lost");
    EXPECT_TRUE(r.events_of("shot_detected").empty());
}

// IP-LIV-10
TEST(LiveStart, IP_LIV_10_SmallMovement) {
    Rig r;
    paper_moves(r, 1.2, false);
    EXPECT_EQ(r.pipeline.status().state, "calibrated");
    EXPECT_EQ(r.events_of("calibration_changed").size(), 1u);
}

// IP-LIV-11
TEST(LiveStart, IP_LIV_11_PaperTakenOffAndPutBack) {
    Rig r;
    Scene s = with_holes({{80, 100}, {120, 100}, {100, 70}});
    r.pipeline.start(s.target, s.bullet_mm);
    r.feed(render(s), 31);
    Scene off = s;
    off.paper = false;
    r.feed(render(off), 65);
    EXPECT_EQ(r.pipeline.status().state, "lost");
    Scene back = s;
    back.holes_mm.push_back({100, 130});
    r.feed(render(back), 3);
    EXPECT_EQ(r.pipeline.status().state, "calibrated");
    ASSERT_EQ(r.state.live->holes.size(), 4u);
    for (auto& h : r.state.live->holes) EXPECT_FALSE(h.counted);
    EXPECT_TRUE(r.events_of("paper_replaced").empty());
}
