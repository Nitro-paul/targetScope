#include <gtest/gtest.h>

#include "process_support.hpp"
#include "v4l2_devices.hpp"

// CR-SRC
// doc/testing/camera_reader.md#source-selection-integration

using namespace camera_reader::testing;
using namespace std::chrono_literals;

// CR-SRC-01
TEST(SourceSelection, CR_SRC_01_NoCamera) {
    ProcessEnv env("none");
    targetscope::frames::FrameSubscriber sub("targetscope/frames/raw");
    env.start();
    auto s = env.status();
    EXPECT_EQ(s["state"], "disabled");
    EXPECT_EQ(s["camera_source"], "none");
    auto e = env.events->expect("camera_state_changed", 3s);
    EXPECT_EQ(e["state"], "disabled");
    EXPECT_FALSE(sub.wait(300ms));
}

// CR-SRC-02
TEST(SourceSelection, CR_SRC_02_FirstCamera) {
    auto devices = camera_reader::list_capture_devices();
    if (devices.empty()) GTEST_SKIP() << "no camera on this PC";
    ProcessEnv env("v4l2");
    env.start();
    ASSERT_TRUE(env.wait_state("connected"));
    auto s = env.status();
    EXPECT_EQ(s["device"], devices.front().path);
    EXPECT_EQ(s["camera_key"], devices.front().bus_info);
    EXPECT_EQ(s["width_px"], 640);
    EXPECT_EQ(s["height_px"], 480);
}

// CR-SRC-03
TEST(SourceSelection, CR_SRC_03_CameraByBusInfo) {
    auto devices = camera_reader::list_capture_devices();
    if (devices.empty()) GTEST_SKIP() << "no camera on this PC";
    ProcessEnv env("v4l2");
    env.put("camera_device", devices.back().bus_info);
    env.start();
    ASSERT_TRUE(env.wait_state("connected"));
    EXPECT_EQ(env.status()["device"], *camera_reader::resolve_device(devices.back().bus_info));
}

// CR-SRC-04
TEST(SourceSelection, CR_SRC_04_CameraNotPresent) {
    ProcessEnv env("v4l2");
    env.put("camera_device", "usb-0000:ff:ff.0-9");
    env.start();
    auto e = env.events->expect("camera_state_changed", 3s);
    EXPECT_EQ(e["state"], "disconnected");
    EXPECT_NE(e["message"].get<std::string>().find("usb-0000:ff:ff.0-9"), std::string::npos);
    EXPECT_EQ(env.status()["state"], "disconnected");
}

// CR-SRC-05
TEST(SourceSelection, CR_SRC_05_NetworkCameraWithoutUrl) {
    ProcessEnv env("rtsp");
    env.start();
    auto e = env.events->expect("camera_state_changed", 3s);
    EXPECT_EQ(e["state"], "disconnected");
    EXPECT_NE(e["message"].get<std::string>().find("camera_rtsp_url"), std::string::npos);
}

// CR-SRC-06
TEST(SourceSelection, CR_SRC_06_NetworkCamera) { GTEST_SKIP() << "needs mediamtx (not installed)"; }
