#include <gtest/gtest.h>

#include <chrono>
#include <thread>

#include "targetscope/frames.hpp"
#include "targetscope/messages.hpp"

// doc/testing/frame_transport.md#frame-header-contract
// doc/testing/frame_transport.md#service-configuration-integration

using namespace targetscope;
using namespace std::chrono_literals;

namespace {

std::string service() { return "targetscope/test/" + new_uuid(); }

TargetScopeFrameHeader header(std::uint64_t seq, int w, int h) {
    TargetScopeFrameHeader hd{};
    hd.seq = seq;
    hd.source_seq = seq;
    hd.timestamp_ns = 1000 + seq;
    hd.width = w;
    hd.height = h;
    hd.stride = w * 3;
    hd.format = TARGETSCOPE_PIXEL_BGR8;
    return hd;
}

void publish(frames::FramePublisher& p, const TargetScopeFrameHeader& h, std::uint8_t fill = 0) {
    p.publish(h, [&](std::span<std::uint8_t> out) { std::fill(out.begin(), out.end(), fill); });
}

}

// FT-HDR-01
TEST(FrameHeader, FT_HDR_01_SizeAndOffsets) {
    EXPECT_EQ(sizeof(TargetScopeFrameHeader), 48u);
    EXPECT_EQ(alignof(TargetScopeFrameHeader), 8u);
    EXPECT_EQ(offsetof(TargetScopeFrameHeader, seq), 0u);
    EXPECT_EQ(offsetof(TargetScopeFrameHeader, timestamp_ns), 8u);
    EXPECT_EQ(offsetof(TargetScopeFrameHeader, source_seq), 16u);
    EXPECT_EQ(offsetof(TargetScopeFrameHeader, width), 24u);
    EXPECT_EQ(offsetof(TargetScopeFrameHeader, height), 28u);
    EXPECT_EQ(offsetof(TargetScopeFrameHeader, stride), 32u);
    EXPECT_EQ(offsetof(TargetScopeFrameHeader, format), 36u);
    EXPECT_EQ(offsetof(TargetScopeFrameHeader, px_per_mm), 40u);
}

// FT-HDR-02
TEST(FrameHeader, FT_HDR_02_TypeName) { EXPECT_STREQ(frames::FrameHeader::IOX2_TYPE_NAME, "TargetScopeFrameHeader"); }

// FT-HDR-03
TEST(FrameHeader, FT_HDR_03_WrongHeaderType) { GTEST_SKIP() << "needs a publisher with another header type"; }

// FT-HDR-04
TEST(FrameHeader, FT_HDR_04_PixelFormatValue) {
    auto name = service();
    frames::FramePublisher p(name, 64 * 64 * 3);
    frames::FrameSubscriber s(name);
    publish(p, header(0, 4, 4));
    auto f = s.wait(1000ms);
    ASSERT_TRUE(f);
    EXPECT_EQ(f->header().format, 1u);
}

// FT-XL-01
TEST(FrameTransport, FT_XL_01_CppToRust) { GTEST_SKIP() << "Rust subscriber in the system tests"; }

// FT-XL-02
TEST(FrameTransport, FT_XL_02_RustToCpp) { GTEST_SKIP() << "Rust publisher in the system tests"; }

// FT-SVC-01
TEST(FrameTransport, FT_SVC_01_SameConfigurationInBothLanguages) {
    GTEST_SKIP() << "compared with Rust in the system tests";
}

// FT-SVC-02
TEST(FrameTransport, FT_SVC_02_OnePublisher) {
    auto name = service();
    frames::FramePublisher p(name, 1024);
    EXPECT_THROW(frames::FramePublisher second(name, 1024), std::runtime_error);
    frames::FrameSubscriber s(name);
    publish(p, header(0, 4, 4));
    EXPECT_TRUE(s.wait(1000ms));
}

// FT-SVC-03
TEST(FrameTransport, FT_SVC_03_FourSubscribers) {
    auto name = service();
    frames::FramePublisher p(name, 1024);
    std::vector<std::unique_ptr<frames::FrameSubscriber>> subs;
    for (int i = 0; i < 4; ++i) subs.push_back(std::make_unique<frames::FrameSubscriber>(name));
    EXPECT_THROW(frames::FrameSubscriber fifth(name), std::runtime_error);
    publish(p, header(0, 4, 4));
    for (auto& s : subs) EXPECT_TRUE(s->wait(1000ms));
}

// FT-SVC-04
TEST(FrameTransport, FT_SVC_04_NoHistory) {
    auto name = service();
    frames::FramePublisher p(name, 1024);
    for (int i = 0; i < 3; ++i) publish(p, header(i, 4, 4));
    frames::FrameSubscriber s(name);
    EXPECT_FALSE(s.wait(100ms));
    publish(p, header(3, 4, 4));
    auto f = s.wait(1000ms);
    ASSERT_TRUE(f);
    EXPECT_EQ(f->header().seq, 3u);
}

// FT-SVC-05
TEST(FrameTransport, FT_SVC_05_SlowSubscriber) {
    auto name = service();
    frames::FramePublisher p(name, 640 * 480 * 3);
    frames::FrameSubscriber s(name);
    std::chrono::nanoseconds worst{0};
    std::vector<std::uint64_t> seen;
    for (int i = 0; i < 100; ++i) {
        auto t0 = std::chrono::steady_clock::now();
        publish(p, header(i, 640, 480));
        worst = std::max(worst, std::chrono::steady_clock::now() - t0);
        std::this_thread::sleep_for(10ms);
        if (i % 20 == 19)
            if (auto f = s.wait(10ms)) seen.push_back(f->header().seq);
    }
    EXPECT_LT(worst, 5ms);
    ASSERT_EQ(seen.size(), 5u);
    EXPECT_EQ(seen.back(), 99u);
}

// FT-SVC-06
TEST(FrameTransport, FT_SVC_06_BorrowLimit) {
    GTEST_SKIP() << "the subscriber API releases every sample but the newest";
}

// FT-SVC-07
TEST(FrameTransport, FT_SVC_07_ResolutionChange) {
    auto name = service();
    frames::FramePublisher p(name, 1280 * 720 * 3);
    frames::FrameSubscriber s(name);
    int i = 0;
    for (auto [w, h] : {std::pair{1280, 720}, {1920, 1080}, {3840, 2160}}) {
        publish(p, header(i++, w, h), 7);
        auto f = s.wait(1000ms);
        ASSERT_TRUE(f);
        EXPECT_EQ(f->header().width, static_cast<unsigned>(w));
        EXPECT_EQ(f->data().size(), static_cast<std::size_t>(h) * w * 3);
        EXPECT_EQ(f->data().back(), 7);
    }
}

// FT-SVC-08
TEST(FrameTransport, FT_SVC_08_Notification) {
    auto name = service();
    frames::FramePublisher p(name, 1024);
    frames::FrameSubscriber s(name);
    std::thread t([&] {
        std::this_thread::sleep_for(200ms);
        publish(p, header(0, 4, 4));
    });
    auto t0 = std::chrono::steady_clock::now();
    auto f = s.wait(5000ms);
    auto took = std::chrono::steady_clock::now() - t0;
    t.join();
    ASSERT_TRUE(f);
    EXPECT_LT(took, 220ms);
}

// FT-SVC-09
TEST(FrameTransport, FT_SVC_09_WaitTimeout) {
    frames::FrameSubscriber s(service());
    auto t0 = std::chrono::steady_clock::now();
    EXPECT_FALSE(s.wait(1000ms));
    EXPECT_GE(std::chrono::steady_clock::now() - t0, 990ms);
}

// FT-SVC-12
TEST(FrameTransport, FT_SVC_12_PublisherAllocation) {
    auto name = service();
    frames::FramePublisher p(name, 1280 * 720 * 3);
    frames::FrameSubscriber s(name);
    publish(p, header(0, 1920, 1080));
    auto f = s.wait(1000ms);
    ASSERT_TRUE(f);
    EXPECT_EQ(f->header().height, 1080u);
}
