#include <gtest/gtest.h>

#include "process_support.hpp"
#include "v4l2_devices.hpp"

// CR-ERR
// doc/testing/camera_reader.md#error-handling-integration

using namespace camera_reader::testing;
using namespace std::chrono_literals;

// CR-ERR-01
TEST(Errors, CR_ERR_01_RedisDown) {
    if (camera_reader::list_capture_devices().empty()) GTEST_SKIP() << "no camera on this PC";
    ProcessEnv env("v4l2");
    env.start();
    ASSERT_TRUE(env.wait_state("connected"));
    targetscope::frames::FrameSubscriber sub("targetscope/frames/raw");
    ASSERT_TRUE(sub.wait(2000ms));
    env.redis.stop();
    int frames = 0;
    auto end = std::chrono::steady_clock::now() + 3s;
    while (std::chrono::steady_clock::now() < end)
        if (sub.wait(200ms)) ++frames;
    EXPECT_GT(frames, 30);
    env.redis.restart();
    bool answered = false;
    for (int i = 0; i < 40 && !answered; ++i) {
        try {
            env.test.send("camera_reader", "camera_status", json::object(), {}, 2s);
            answered = true;
        } catch (const targetscope::BusError&) {
        }
    }
    EXPECT_TRUE(answered);
}

// CR-ERR-02
TEST(Errors, CR_ERR_02_Shutdown) {
    ProcessEnv env("none");
    env.start();
    env.process->terminate();
    EXPECT_TRUE(env.process->wait_exit(2s));
    EXPECT_EQ(env.process->exit_code(), 0);
}
