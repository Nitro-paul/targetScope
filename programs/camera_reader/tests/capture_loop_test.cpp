#include <gtest/gtest.h>

#include <thread>

#include "loop_support.hpp"

// CR-CAP
// doc/testing/camera_reader.md#capture-loop-and-frames-unit-and-integration

using namespace camera_reader;
using namespace camera_reader::testing;
using namespace std::chrono_literals;

namespace {

struct Rig {
    std::shared_ptr<FakeCamera> camera = std::make_shared<FakeCamera>();
    Recorder recorder;
    std::string service = unique_service();
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
              [this](const CameraStatus& s) { recorder(s); }, service, fast_timing()) {}
};

}

// CR-CAP-01
TEST(CaptureLoop, CR_CAP_01_HeaderFields) {
    Rig r;
    targetscope::frames::FrameSubscriber sub(r.service);
    r.loop.start();
    std::vector<TargetScopeFrameHeader> headers;
    auto deadline = std::chrono::steady_clock::now() + 10s;
    while (headers.size() < 100 && std::chrono::steady_clock::now() < deadline) {
        if (auto f = sub.wait(500ms)) headers.push_back(f->header());
    }
    ASSERT_GE(headers.size(), 100u);
    for (size_t i = 1; i < headers.size(); ++i) {
        EXPECT_EQ(headers[i].seq, headers[i - 1].seq + 1);
        EXPECT_GT(headers[i].timestamp_ns, headers[i - 1].timestamp_ns);
    }
    EXPECT_EQ(headers[0].seq, 0u);
    for (auto& h : headers) {
        EXPECT_EQ(h.source_seq, h.seq);
        EXPECT_EQ(h.width, 640u);
        EXPECT_EQ(h.height, 480u);
        EXPECT_EQ(h.stride, 1920u);
        EXPECT_EQ(h.format, 1u);
        EXPECT_EQ(h.px_per_mm, 0.0);
    }
}

// CR-CAP-02
TEST(CaptureLoop, CR_CAP_02_PixelsUnchanged) {
    Rig r;
    targetscope::frames::FrameSubscriber sub(r.service);
    r.loop.start();
    int checked = 0;
    for (int i = 0; i < 20; ++i) {
        auto f = sub.wait(500ms);
        ASSERT_TRUE(f);
        cv::Mat expected = FakeCamera::pattern(640, 480, static_cast<long>(f->header().seq));
        ASSERT_EQ(f->data().size(), expected.total() * 3);
        EXPECT_EQ(std::memcmp(f->data().data(), expected.data, f->data().size()), 0);
        ++checked;
    }
    EXPECT_EQ(checked, 20);
}

// CR-CAP-03
TEST(CaptureLoop, CR_CAP_03_YuyvConversion) { GTEST_SKIP() << "needs a v4l2loopback device (kernel module, root)"; }

// CR-CAP-04
TEST(CaptureLoop, CR_CAP_04_MjpegConversion) { GTEST_SKIP() << "needs a v4l2loopback device (kernel module, root)"; }

// CR-CAP-05
TEST(CaptureLoop, CR_CAP_05_ResolutionNotAvailable) {
    Rig r;
    r.camera->info = {1280, 720, 50.0};
    targetscope::frames::FrameSubscriber sub(r.service);
    r.loop.start();
    auto f = sub.wait(1000ms);
    ASSERT_TRUE(f);
    EXPECT_EQ(f->header().width, 1280u);
    EXPECT_EQ(f->header().height, 720u);
    EXPECT_EQ(r.loop.status().info->width_px, 1280);
}

// CR-CAP-06
TEST(CaptureLoop, CR_CAP_06_NobodyWatching) {
    Rig r;
    r.loop.start();
    ASSERT_TRUE(eventually([&] { return r.loop.status().frames_published >= 10; }));
    EXPECT_EQ(r.loop.status().state, CameraState::Connected);
}

// CR-CAP-07
TEST(CaptureLoop, CR_CAP_07_FrameCounter) {
    Rig r;
    r.loop.start();
    ASSERT_TRUE(eventually([&] { return r.loop.status().frames_published >= 5; }));
    auto a = r.loop.status().frames_published;
    std::this_thread::sleep_for(1s);
    auto b = r.loop.status().frames_published;
    EXPECT_NEAR(static_cast<double>(b - a), 50.0, 8.0);
}

// CR-CAP-08
TEST(CaptureLoop, CR_CAP_08_PublisherSize) {
    targetscope::frames::FramePublisher p(unique_service(), 1280 * 720 * 3);
    EXPECT_EQ(p.initial_max_len(), 1280u * 720u * 3u);
}
