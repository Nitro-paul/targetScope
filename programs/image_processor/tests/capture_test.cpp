#include "capture.hpp"

#include <gtest/gtest.h>

#include <filesystem>
#include <opencv2/imgcodecs.hpp>

#include "rig.hpp"

// IP-CAP
// doc/testing/image_processor.md#capture-integration

using namespace image_processor;
using namespace image_processor::testing;
using targetscope::ErrorCode;
using targetscope::ServiceError;

// IP-CAP-01
TEST(Capture, IP_CAP_01_TargetImage) {
    Rig r;
    Scene s;
    s.holes_mm = {{80, 100}, {120, 105}};
    r.pipeline.start(s.target, 4.0);
    r.feed(render(s), 3);
    auto name = capture_target_image(r.state, r.dirs.staging_dir, 2400);
    EXPECT_EQ(name.rfind("target_", 0), 0u);
    cv::Mat img = cv::imread((r.dirs.staging_dir / name).string());
    ASSERT_FALSE(img.empty());
    EXPECT_EQ(img.size(), cv::Size(2400, 2400));
    for (auto& f : std::filesystem::directory_iterator(r.dirs.staging_dir)) EXPECT_NE(f.path().extension(), ".part");
    cv::Mat ideal;
    cv::resize(render_paper(s, 8.0), ideal, img.size(), 0, 0, cv::INTER_AREA);
    cv::Mat diff;
    cv::absdiff(img, ideal, diff);
    EXPECT_LT(cv::mean(diff)[0], 6.0);
}

// IP-CAP-02
TEST(Capture, IP_CAP_02_HandInFrontOfThePaper) {
    Rig r;
    Scene s;
    r.pipeline.start(s.target, 4.0);
    r.feed(render(s), 3);
    Scene hand = s;
    hand.hand_fraction = 0.3;
    r.feed(render(hand), 5);
    auto name = capture_target_image(r.state, r.dirs.staging_dir, 1000);
    cv::Mat img = cv::imread((r.dirs.staging_dir / name).string());
    cv::Scalar top = cv::mean(img(cv::Rect(100, 20, 800, 100)));
    EXPECT_GT(top[2], 200);
    EXPECT_GT(top[0], 200);
}

// IP-CAP-03
TEST(Capture, IP_CAP_03_NoGoodFrame) {
    Rig r;
    try {
        capture_target_image(r.state, r.dirs.staging_dir, 2400);
        FAIL();
    } catch (const ServiceError& e) {
        EXPECT_EQ(e.code, ErrorCode::Unavailable);
    }
    r.pipeline.start(tt_ref(), 4.0);
    try {
        capture_target_image(r.state, r.dirs.staging_dir, 2400);
        FAIL();
    } catch (const ServiceError& e) {
        EXPECT_EQ(e.code, ErrorCode::Unavailable);
    }
    EXPECT_TRUE(std::filesystem::is_empty(r.dirs.staging_dir));
}

// IP-CAP-04
TEST(Capture, IP_CAP_04_SizeSetting) {
    Rig r;
    Scene s;
    r.pipeline.start(s.target, 4.0);
    r.feed(render(s), 2);
    auto name = capture_target_image(r.state, r.dirs.staging_dir, 1000);
    EXPECT_EQ(cv::imread((r.dirs.staging_dir / name).string()).size(), cv::Size(1000, 1000));
}

// IP-CAP-05
TEST(Capture, IP_CAP_05_CameraPicture) {
    Rig r;
    Scene s;
    r.state.newest_raw = render(s);
    r.state.last_frame_time = std::chrono::steady_clock::now();
    auto [name, corrected] = capture_raw_image(r.state, r.dirs.staging_dir);
    EXPECT_EQ(name.rfind("raw_", 0), 0u);
    EXPECT_FALSE(corrected);
    EXPECT_EQ(cv::imread((r.dirs.staging_dir / name).string()).size(), cv::Size(1920, 1080));
}

// IP-CAP-06
TEST(Capture, IP_CAP_06_CameraPictureWithLensCorrection) {
    Rig r;
    Scene s;
    s.image = cv::Size(1280, 720);
    r.install_lens(-0.15, s.image);
    r.state.newest_raw = render(s);
    r.state.last_frame_time = std::chrono::steady_clock::now();
    auto [name, corrected] = capture_raw_image(r.state, r.dirs.staging_dir);
    EXPECT_TRUE(corrected);
}

// IP-CAP-07
TEST(Capture, IP_CAP_07_NoFrames) {
    Rig r;
    try {
        capture_raw_image(r.state, r.dirs.staging_dir);
        FAIL();
    } catch (const ServiceError& e) {
        EXPECT_EQ(e.code, ErrorCode::Unavailable);
    }
}
