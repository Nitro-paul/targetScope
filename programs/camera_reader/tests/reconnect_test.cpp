#include <gtest/gtest.h>

#include "loop_support.hpp"

// CR-REC
// doc/testing/camera_reader.md#reconnecting-unit-and-integration

using namespace camera_reader;
using namespace camera_reader::testing;
using namespace std::chrono_literals;

namespace {

struct Rig {
    std::shared_ptr<FakeCamera> camera = std::make_shared<FakeCamera>();
    Recorder recorder;
    CaptureLoop loop;
    Rig()
        : loop(
              [this] {
                  SourceChoice c;
                  c.camera_source = "v4l2";
                  c.camera_key = "fake-bus";
                  c.source = std::make_unique<FakeSource>(camera);
                  return c;
              },
              [this](const CameraStatus& s) { recorder(s); }, unique_service(), fast_timing()) {}
};

}

// CR-REC-01
TEST(Reconnect, CR_REC_01_ReadErrors) {
    Rig r;
    {
        std::lock_guard l(r.camera->mutex);
        for (int i = 0; i < 10; ++i) r.camera->script.push_back({.fail = true});
    }
    r.loop.start();
    ASSERT_TRUE(eventually([&] { return r.camera->opens >= 2; }));
    ASSERT_TRUE(eventually(
        [&] { return r.loop.status().state == CameraState::Connected && r.loop.status().frames_published > 3; }));
    EXPECT_GE(r.camera->closes, 1);
    auto states = r.recorder.get();
    ASSERT_GE(states.size(), 3u);
    EXPECT_EQ(states[0].state, CameraState::Connected);
    EXPECT_EQ(states[1].state, CameraState::Disconnected);
    EXPECT_EQ(states[2].state, CameraState::Connected);
}

// CR-REC-02
TEST(Reconnect, CR_REC_02_NineReadErrors) {
    Rig r;
    {
        std::lock_guard l(r.camera->mutex);
        for (int i = 0; i < 9; ++i) r.camera->script.push_back({.fail = true});
    }
    r.loop.start();
    ASSERT_TRUE(eventually([&] { return r.loop.status().frames_published > 5; }));
    EXPECT_EQ(r.camera->opens, 1);
    EXPECT_EQ(r.recorder.get().size(), 1u);
}

// CR-REC-03
TEST(Reconnect, CR_REC_03_NoFramesFor2S) {
    Rig r;
    r.loop.start();
    ASSERT_TRUE(eventually([&] { return r.loop.status().frames_published > 3; }));
    {
        std::lock_guard l(r.camera->mutex);
        r.camera->script.push_back({.stall = 1500ms});
    }
    ASSERT_TRUE(eventually([&] { return r.loop.status().state == CameraState::Disconnected; }, 1200ms));
    ASSERT_TRUE(eventually([&] { return r.loop.status().state == CameraState::Connected; }, 5000ms));
    EXPECT_GE(r.camera->opens, 2);
}

// CR-REC-04
TEST(Reconnect, CR_REC_04_NetworkStreamStops) { GTEST_SKIP() << "needs mediamtx (not installed)"; }

// CR-REC-05
TEST(Reconnect, CR_REC_05_EveryStateChangeIsSent) {
    Rig r;
    r.camera->fail_open = 1;
    {
        std::lock_guard l(r.camera->mutex);
        for (int i = 0; i < 10; ++i) r.camera->script.push_back({.fail = true});
    }
    r.loop.start();
    ASSERT_TRUE(eventually([&] { return r.loop.status().state == CameraState::Connected && r.camera->opens >= 3; }));
    auto states = r.recorder.get();
    ASSERT_GE(states.size(), 3u);
    for (size_t i = 1; i < states.size(); ++i)
        EXPECT_FALSE(states[i].state == states[i - 1].state && states[i].message == states[i - 1].message);
    EXPECT_EQ(states[0].state, CameraState::Disconnected);
    for (auto& s : states) EXPECT_EQ(s.camera_key, "fake-bus");
    for (auto& s : states)
        if (s.state == CameraState::Connected) {
            ASSERT_TRUE(s.info);
            EXPECT_EQ(s.info->width_px, 640);
        }
}
