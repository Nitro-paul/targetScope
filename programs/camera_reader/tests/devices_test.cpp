#include <gtest/gtest.h>

#include "camera_identity.hpp"
#include "process_support.hpp"
#include "v4l2_devices.hpp"

// doc/testing/camera_reader.md#devices-and-camera-identity-integration

using namespace camera_reader::testing;

// CR-DEV-01
TEST(Devices, CR_DEV_01_DeviceList) {
    auto devices = camera_reader::list_capture_devices();
    if (devices.empty()) GTEST_SKIP() << "no camera on this PC";
    ProcessEnv env("none");
    env.start();
    auto r = env.call("camera_list_devices");
    ASSERT_EQ(r["devices"].size(), devices.size());
    for (auto& d : r["devices"]) {
        EXPECT_FALSE(d["path"].get<std::string>().empty());
        EXPECT_FALSE(d["name"].get<std::string>().empty());
        EXPECT_FALSE(d["bus_info"].get<std::string>().empty());
        ASSERT_FALSE(d["formats"].empty());
        for (auto& f : d["formats"]) {
            EXPECT_GT(f["width_px"].get<int>(), 0);
            EXPECT_GT(f["height_px"].get<int>(), 0);
            EXPECT_FALSE(f["fps"].empty());
        }
    }
}

// CR-DEV-02
TEST(Devices, CR_DEV_02_NoCameras) {
    if (!camera_reader::list_capture_devices().empty()) GTEST_SKIP() << "this PC has a camera";
    ProcessEnv env("none");
    env.start();
    EXPECT_TRUE(env.call("camera_list_devices")["devices"].empty());
}

// CR-DEV-03
TEST(Devices, CR_DEV_03_IdentityOfAUsbCamera) {
    auto devices = camera_reader::list_capture_devices();
    if (devices.empty()) GTEST_SKIP() << "no camera on this PC";
    ProcessEnv env("v4l2");
    env.start();
    ASSERT_TRUE(env.wait_state("connected"));
    auto s = env.status();
    EXPECT_EQ(s["camera_key"], devices.front().bus_info);
    auto e = env.events->expect("camera_state_changed", 3s);
    EXPECT_EQ(e["camera_key"], devices.front().bus_info);
    EXPECT_EQ(e["width_px"], 640);
}

// CR-DEV-04
TEST(Devices, CR_DEV_04_IdentityOfANetworkCamera) {
    EXPECT_EQ(camera_reader::camera_key_rtsp("rtsp://cam.local/stream"), "rtsp://cam.local/stream");
    EXPECT_EQ(camera_reader::camera_key_v4l2("usb-0000:00:14.0-2"), "usb-0000:00:14.0-2");
}
