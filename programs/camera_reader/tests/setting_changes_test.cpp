#include <gtest/gtest.h>

#include <atomic>

#include "loop_support.hpp"

// CR-SET
// doc/testing/camera_reader.md#setting-changes-integration

using namespace camera_reader;
using namespace camera_reader::testing;
using namespace std::chrono_literals;

namespace {

struct Rig {
    std::shared_ptr<FakeCamera> camera = std::make_shared<FakeCamera>();
    Recorder recorder;
    std::atomic<bool> none{false};
    CaptureLoop loop;
    Rig()
        : loop(
              [this] {
                  SourceChoice c;
                  c.camera_key = "fake-bus";
                  if (none) {
                      c.camera_source = "none";
                      c.message = "camera_source is none";
                      return c;
                  }
                  c.camera_source = "v4l2";
                  c.source = std::make_unique<FakeSource>(camera);
                  return c;
              },
              [this](const CameraStatus& s) { recorder(s); }, unique_service(), fast_timing()) {}
};

}

// CR-SET-01
TEST(SettingChanges, CR_SET_01_FrameRate) {
    Rig r;
    r.loop.start();
    ASSERT_TRUE(eventually([&] { return r.loop.status().frames_published > 3; }));
    {
        std::lock_guard l(r.camera->mutex);
        r.camera->info.fps = 15;
    }
    r.loop.reopen();
    ASSERT_TRUE(
        eventually([&] { return r.camera->opens == 2 && r.loop.status().info && r.loop.status().info->fps == 15; }));
}

// CR-SET-02
TEST(SettingChanges, CR_SET_02_SourceOff) {
    Rig r;
    r.loop.start();
    ASSERT_TRUE(eventually([&] { return r.loop.status().frames_published > 3; }));
    r.none = true;
    r.loop.reopen();
    ASSERT_TRUE(eventually([&] { return r.loop.status().state == CameraState::Disabled; }));
    auto n = r.loop.status().frames_published;
    std::this_thread::sleep_for(300ms);
    EXPECT_EQ(r.loop.status().frames_published, n);
}

// CR-SET-03
TEST(SettingChanges, CR_SET_03_EachReopenSetting) {
    Rig r;
    r.loop.start();
    ASSERT_TRUE(eventually([&] { return r.camera->opens == 1 && r.loop.status().frames_published > 1; }));
    for (int i = 2; i <= 7; ++i) {
        r.loop.reopen();
        ASSERT_TRUE(eventually([&] { return r.camera->opens == i; })) << "reopen " << i;
        ASSERT_TRUE(eventually([&] { return r.camera->closes == i - 1; }));
    }
}

// CR-SET-04
TEST(SettingChanges, CR_SET_04_StoredControlValues) {
    GTEST_SKIP() << "covered by the process test CR_CTL_03 on a real camera";
}

// CR-SET-05
TEST(SettingChanges, CR_SET_05_ControlsReset) { GTEST_SKIP() << "needs a camera with controls (vivid)"; }

// CR-SET-06
TEST(SettingChanges, CR_SET_06_OtherSettings) {
    Rig r;
    r.loop.start();
    ASSERT_TRUE(eventually([&] { return r.loop.status().frames_published > 3; }));
    std::this_thread::sleep_for(300ms);
    EXPECT_EQ(r.camera->opens, 1);
}
