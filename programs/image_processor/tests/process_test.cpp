#include <gtest/gtest.h>

#include <filesystem>
#include <opencv2/imgcodecs.hpp>

#include "process_support.hpp"

// doc/testing/image_processor.md#setup

using namespace image_processor;
using namespace image_processor::testing;
using targetscope::BusError;
using targetscope::ErrorCode;

namespace {

Scene scene(std::vector<cv::Point2d> holes = {{80, 100}, {120, 105}, {100, 70}}) {
    Scene s;
    s.holes_mm = std::move(holes);
    return s;
}

int count_live(targetscope::frames::FrameSubscriber& sub, std::chrono::milliseconds span) {
    int n = 0;
    auto end = std::chrono::steady_clock::now() + span;
    while (std::chrono::steady_clock::now() < end)
        if (sub.wait(50ms)) ++n;
    return n;
}

}

TEST(Process, IP_LIV_01_StartWithoutFrames) {
    ProcessEnv env;
    env.start();
    env.feeder.reset();
    targetscope::frames::FrameSubscriber live("targetscope/frames/live");
    auto status = env.live_start(scene());
    EXPECT_EQ(status["state"], "not_calibrated");
    EXPECT_EQ(status["message"], "no camera frames");
    EXPECT_FALSE(live.wait(300ms));
}

TEST(Process, IP_PRC_01_AutomaticCalibrationAndLiveFrames) {
    ProcessEnv env;
    Scene s = scene();
    env.camera.set(render(s));
    env.start();
    targetscope::frames::FrameSubscriber live("targetscope/frames/live");
    env.live_start(s);
    auto changed = env.events->expect("calibration_changed", 3s);
    EXPECT_EQ(changed["state"], "calibrated");
    EXPECT_EQ(changed["method"], "auto");
    int checked = 0;
    for (int i = 0; i < 30 && checked < 5; ++i) {
        auto f = live.wait(500ms);
        if (!f) continue;
        EXPECT_EQ(f->header().width, 1200u);
        EXPECT_EQ(f->header().height, 1200u);
        EXPECT_EQ(f->header().stride, 3600u);
        EXPECT_EQ(f->header().format, 1u);
        EXPECT_DOUBLE_EQ(f->header().px_per_mm, 6.0);
        if (auto ts = env.feeder->timestamp_of(f->header().source_seq)) {
            EXPECT_EQ(f->header().timestamp_ns, *ts);
            ++checked;
        }
    }
    EXPECT_GE(checked, 3);
}

TEST(Process, IP_PRC_02_NewShotIsReported) {
    ProcessEnv env;
    Scene s = scene();
    env.camera.set(render(s));
    env.start();
    env.live_start(s);
    ASSERT_TRUE(env.wait_state("calibrated"));
    env.call("live_set_counting", {{"counting", true}});
    s.holes_mm.push_back({100, 130});
    env.camera.set(render(s));
    auto shot = env.events->expect("shot_detected", 4s);
    EXPECT_EQ(shot["live_paper"], 1);
    EXPECT_GT(shot["confidence"].get<double>(), 0.5);
    std::this_thread::sleep_for(1s);
    for (auto& [type, payload] : env.events->drain()) EXPECT_NE(type, "shot_detected");
}

TEST(Process, IP_PRC_03_Stop) {
    ProcessEnv env;
    Scene s = scene();
    env.camera.set(render(s));
    env.start();
    targetscope::frames::FrameSubscriber live("targetscope/frames/live");
    env.live_start(s);
    ASSERT_TRUE(env.wait_state("calibrated"));
    EXPECT_GT(count_live(live, 500ms), 3);
    env.call("live_stop");
    count_live(live, 200ms);
    EXPECT_EQ(count_live(live, 500ms), 0);
    auto status = env.call("live_calibration_status");
    EXPECT_EQ(status["state"], "not_calibrated");
    EXPECT_EQ(status["message"], "live mode not active");
}

TEST(Process, IP_PRC_04_CaptureTargetImage) {
    ProcessEnv env;
    Scene s = scene();
    env.camera.set(render(s));
    env.start();
    env.live_start(s);
    ASSERT_TRUE(env.wait_state("calibrated"));
    auto r = env.call("capture_target_image");
    const std::string name = r["staged_file"];
    EXPECT_EQ(name.rfind("target_", 0), 0u);
    EXPECT_EQ(cv::imread((env.dirs.staging_dir / name).string()).size(), cv::Size(2400, 2400));
    auto raw = env.call("capture_raw_image");
    EXPECT_EQ(raw["lens_corrected"], false);
    EXPECT_EQ(cv::imread((env.dirs.staging_dir / raw["staged_file"].get<std::string>()).string()).size(),
              cv::Size(1920, 1080));
}

TEST(Process, IP_PRC_05_PhotosDuringLiveMode) {
    ProcessEnv env;
    Scene s = scene();
    env.camera.set(render(s));
    env.start();
    targetscope::frames::FrameSubscriber live("targetscope/frames/live");
    env.live_start(s);
    ASSERT_TRUE(env.wait_state("calibrated"));
    count_live(live, 300ms);
    const int before = count_live(live, 2000ms);
    Scene photo = scene({{80, 100}, {120, 105}, {100, 70}, {60, 60}});
    photo.image = cv::Size(4000, 3000);
    const std::string file = "upload_photo.jpg";
    cv::imwrite((env.dirs.staging_dir / file).string(), render(photo));
    std::thread jobs([&] {
        for (int i = 0; i < 2; ++i)
            env.call("detect_shots_in_photo",
                     {{"staged_file", file}, {"target_type", json(s.target)}, {"bullet_diameter_mm", 4.0}});
    });
    const int during = count_live(live, 2000ms);
    jobs.join();
    EXPECT_GT(during, 0.9 * before) << before << " frames before, " << during << " during the photo jobs";
}

TEST(Process, IP_PRC_06_LensCalibrationRequests) {
    ProcessEnv env;
    Scene s = scene();
    env.camera.set(render(s));
    env.start();
    env.live_start(s);
    ASSERT_TRUE(env.wait_state("calibrated"));
    try {
        env.call("lens_calibration_start");
        FAIL();
    } catch (const BusError& e) {
        EXPECT_EQ(e.code, ErrorCode::InvalidState);
    }
    env.call("live_stop");
    EXPECT_EQ(env.call("lens_calibration_start")["views"], 0);
    auto status = env.call("lens_calibration_status");
    EXPECT_EQ(status["collecting_views"], 0);
    EXPECT_EQ(status["camera_key"], "fake-cam");
    EXPECT_EQ(status["calibrated"], false);
    std::this_thread::sleep_for(300ms);
    auto no_board = env.call("lens_calibration_capture");
    EXPECT_EQ(no_board["found"], false);
    EXPECT_EQ(no_board["views"], 0);
    env.call("lens_calibration_cancel");
    EXPECT_TRUE(env.call("lens_calibration_status")["collecting_views"].is_null());
    try {
        env.call("lens_calibration_capture");
        FAIL();
    } catch (const BusError& e) {
        EXPECT_EQ(e.code, ErrorCode::InvalidState);
    }
    {
        std::lock_guard l(env.mutex);
        EXPECT_TRUE(env.saved.empty());
    }
    env.call("lens_calibration_start");
    try {
        env.call("lens_calibration_finish");
        FAIL();
    } catch (const BusError& e) {
        EXPECT_EQ(e.code, ErrorCode::ValidationFailed);
    }
}

// IP-ERR-03
TEST(Process, IP_ERR_03_Shutdown) {
    ProcessEnv env;
    Scene s = scene();
    env.camera.set(render(s));
    env.start();
    env.live_start(s);
    ASSERT_TRUE(env.wait_state("calibrated"));
    env.process->terminate();
    EXPECT_TRUE(env.process->wait_exit(2s));
    EXPECT_EQ(env.process->exit_code(), 0);
}

TEST(Process, IP_ERR_01_FramesStopAndComeBack) {
    ProcessEnv env;
    Scene s = scene();
    env.camera.set(render(s));
    env.start();
    targetscope::frames::FrameSubscriber live("targetscope/frames/live");
    env.live_start(s);
    ASSERT_TRUE(env.wait_state("calibrated"));
    env.feeder->pause();
    std::this_thread::sleep_for(3s);
    auto status = env.call("live_calibration_status");
    EXPECT_EQ(status["state"], "not_calibrated");
    EXPECT_EQ(status["message"], "no camera frames");
    for (auto [type, payload] : {std::pair<std::string, json>{"capture_raw_image", json::object()},
                                 {"live_set_counting", json{{"counting", true}}}}) {
        try {
            env.call(type, payload);
            FAIL() << type;
        } catch (const BusError& e) {
            EXPECT_EQ(e.code, ErrorCode::Unavailable);
        }
    }
    EXPECT_NO_THROW(env.call("capture_target_image"));
    count_live(live, 200ms);
    EXPECT_EQ(count_live(live, 400ms), 0);
    s.holes_mm.push_back({100, 130});
    env.camera.set(render(s));
    env.feeder->resume();
    ASSERT_TRUE(env.wait_state("calibrated"));
    std::this_thread::sleep_for(1500ms);
    for (auto& [type, payload] : env.events->drain()) EXPECT_NE(type, "shot_detected");
}
