#include <gtest/gtest.h>

#include "process_support.hpp"
#include "v4l2_devices.hpp"

// CR-CTL
// doc/testing/camera_reader.md#camera-controls-integration-vivid

using namespace camera_reader::testing;
using namespace std::chrono_literals;

namespace {

struct Camera {
    ProcessEnv env{"v4l2"};
    json controls;
    bool available = false;
    Camera() {
        if (camera_reader::list_capture_devices().empty()) return;
        env.start();
        available = env.wait_state("connected");
        if (available) controls = env.call("camera_list_controls")["controls"];
    }
    json find(const std::string& name_part) {
        for (auto& c : controls)
            if (c["name"].get<std::string>().find(name_part) != std::string::npos) return c;
        return nullptr;
    }
};

json env_set(Camera& cam, const json& id, const json& value) {
    return cam.env.call("camera_set_control", {{"id", id}, {"value", value}});
}

}

// CR-CTL-01
TEST(Controls, CR_CTL_01_List) {
    Camera cam;
    if (!cam.available) GTEST_SKIP() << "no camera on this PC";
    ASSERT_FALSE(cam.controls.empty());
    for (auto& c : cam.controls) {
        auto kind = c["kind"].get<std::string>();
        EXPECT_TRUE(kind == "integer" || kind == "boolean" || kind == "menu" || kind == "button") << kind;
        EXPECT_TRUE(c.contains("min") && c.contains("max") && c.contains("step") && c.contains("default") &&
                    c.contains("value"));
        EXPECT_TRUE(c.contains("inactive"));
        if (kind == "menu") EXPECT_FALSE(c["menu"].empty());
    }
    EXPECT_FALSE(cam.find("Brightness").is_null());
}

// CR-CTL-02
TEST(Controls, CR_CTL_02_InactiveControls) {
    Camera cam;
    if (!cam.available) GTEST_SKIP() << "no camera on this PC";
    auto wb_auto = cam.find("White Balance");
    auto temp = cam.find("Temperature");
    if (wb_auto.is_null() || temp.is_null() || wb_auto["kind"] != "boolean")
        GTEST_SKIP() << "camera has no auto white balance";
    EXPECT_EQ(temp["inactive"], wb_auto["value"] == 1);
}

// CR-CTL-03
TEST(Controls, CR_CTL_03_SetAValue) {
    Camera cam;
    if (!cam.available) GTEST_SKIP() << "no camera on this PC";
    auto b = cam.find("Brightness");
    ASSERT_FALSE(b.is_null());
    auto r = env_set(cam, b["id"], b["value"]);
    EXPECT_EQ(r["controls"].size(), cam.controls.size());
    std::lock_guard l(cam.env.mutex);
    ASSERT_FALSE(cam.env.saved.empty());
    auto stored = json::parse(cam.env.saved.back()["value"].get<std::string>());
    auto key = cam.env.status()["camera_key"].get<std::string>();
    EXPECT_EQ(stored[key][std::to_string(b["id"].get<int>())], b["value"]);
}

// CR-CTL-04
TEST(Controls, CR_CTL_04_OneControlChangesAnother) { GTEST_SKIP() << "would change the camera; needs vivid"; }

// CR-CTL-05
TEST(Controls, CR_CTL_05_OutOfRange) {
    Camera cam;
    if (!cam.available) GTEST_SKIP() << "no camera on this PC";
    auto b = cam.find("Brightness");
    try {
        cam.env.call("camera_set_control", {{"id", b["id"]}, {"value", b["max"].get<int>() + 1}});
        FAIL();
    } catch (const targetscope::BusError& e) {
        EXPECT_EQ(e.code, targetscope::ErrorCode::ValidationFailed);
    }
    EXPECT_TRUE(cam.env.saved.empty());
}

// CR-CTL-06
TEST(Controls, CR_CTL_06_WrongStep) {
    Camera cam;
    if (!cam.available) GTEST_SKIP() << "no camera on this PC";
    for (auto& c : cam.controls)
        if (c["kind"] == "integer" && c["step"].get<int>() > 1) {
            try {
                cam.env.call("camera_set_control", {{"id", c["id"]}, {"value", c["min"].get<int>() + 1}});
                FAIL();
            } catch (const targetscope::BusError& e) {
                EXPECT_EQ(e.code, targetscope::ErrorCode::ValidationFailed);
            }
            EXPECT_TRUE(cam.env.saved.empty());
            return;
        }
    GTEST_SKIP() << "no control with a step above 1";
}

// CR-CTL-07
TEST(Controls, CR_CTL_07_UnknownControl) {
    Camera cam;
    if (!cam.available) GTEST_SKIP() << "no camera on this PC";
    try {
        cam.env.call("camera_set_control", {{"id", 12345}, {"value", 1}});
        FAIL();
    } catch (const targetscope::BusError& e) {
        EXPECT_EQ(e.code, targetscope::ErrorCode::ValidationFailed);
    }
}

// CR-CTL-08
TEST(Controls, CR_CTL_08_NoCamera) {
    ProcessEnv env("none");
    env.start();
    for (auto [type, payload] : {std::pair<std::string, json>{"camera_list_controls", json::object()},
                                 {"camera_set_control", json{{"id", 1}, {"value", 1}}}}) {
        try {
            env.call(type, payload);
            FAIL() << type;
        } catch (const targetscope::BusError& e) {
            EXPECT_EQ(e.code, targetscope::ErrorCode::Unavailable);
        }
    }
}

// CR-CTL-09
TEST(Controls, CR_CTL_09_NetworkCamera) { GTEST_SKIP() << "needs mediamtx (not installed)"; }

// CR-CTL-10
TEST(Controls, CR_CTL_10_ReappliedAfterReopening) { GTEST_SKIP() << "needs vivid (kernel module, root)"; }

// CR-CTL-11
TEST(Controls, CR_CTL_11_StoredValueNoLongerValid) {
    if (camera_reader::list_capture_devices().empty()) GTEST_SKIP() << "no camera on this PC";
    ProcessEnv env("v4l2");
    auto devices = camera_reader::list_capture_devices();
    env.put("camera_controls", json{{devices.front().bus_info, {{"9963776", 99999}}}}.dump());
    env.start();
    ASSERT_TRUE(env.wait_state("connected"));
    bool flowing = false;
    for (int i = 0; i < 50 && !flowing; ++i) {
        std::this_thread::sleep_for(100ms);
        flowing = env.status()["frames_published"].get<int>() > 0;
    }
    EXPECT_TRUE(flowing);
}

// CR-CTL-12
TEST(Controls, CR_CTL_12_ValuesPerCamera) { GTEST_SKIP() << "needs two cameras"; }
