#include <gtest/gtest.h>

#include "rig.hpp"

// IP-FRM
// doc/testing/image_processor.md#live-frames-integration

using namespace image_processor;
using namespace image_processor::testing;

namespace {

double nearest(const cv::Mat& img, cv::Point p, const cv::Scalar& color) {
    double best = 1e9;
    for (int dy = -1; dy <= 1; ++dy)
        for (int dx = -1; dx <= 1; ++dx) {
            auto v = img.at<cv::Vec3b>(p.y + dy, p.x + dx);
            best = std::min(best, cv::norm(cv::Vec3d(v[0], v[1], v[2]) - cv::Vec3d(color[0], color[1], color[2])));
        }
    return best;
}

cv::Point px(const cv::Point2d& mm) {
    return {static_cast<int>(std::lround(mm.x * 6)), static_cast<int>(std::lround(mm.y * 6))};
}

}

// IP-FRM-01
TEST(LiveFrames, IP_FRM_01_OnlyWhenCalibrated) {
    Rig r;
    Scene none;
    none.paper = false;
    r.pipeline.start(none.target, 4.0);
    r.feed(render(none), 3);
    EXPECT_FALSE(r.last_out);
    Scene s;
    r.feed(render(s), 1);
    EXPECT_TRUE(r.last_out);
}

// IP-FRM-02
TEST(LiveFrames, IP_FRM_02_SizeAndScale) {
    Rig r;
    Scene s;
    r.pipeline.start(s.target, 4.0);
    r.feed(render(s));
    ASSERT_TRUE(r.last_out);
    EXPECT_EQ(r.last_out->size(), cv::Size(1200, 1200));
    EXPECT_DOUBLE_EQ(r.pipeline.px_per_mm(), 6.0);
}

// IP-FRM-03
TEST(LiveFrames, IP_FRM_03_SettingChanges) {
    Rig r;
    Scene s;
    r.pipeline.start(s.target, 4.0);
    r.feed(render(s));
    r.state.runtime.live_long_side_px = 800;
    r.pipeline.start(s.target, 4.0);
    r.feed(render(s));
    ASSERT_TRUE(r.last_out);
    EXPECT_EQ(r.last_out->size(), cv::Size(800, 800));
}

// IP-FRM-04
TEST(LiveFrames, IP_FRM_04_Drawing) {
    Rig r;
    Scene s;
    s.holes_mm = {{60, 60}};
    r.start(s, true);
    s.holes_mm.push_back({140, 140});
    cv::Mat frame = render(s);
    r.feed(frame, 12);
    s.holes_mm.push_back({150, 60});
    r.feed(render(s), 2);
    ASSERT_TRUE(r.last_out);
    const cv::Mat& out = *r.last_out;
    const int r_px = 12;
    EXPECT_LT(nearest(out, px({60, 60}) + cv::Point(r_px, 0), cv::Scalar(150, 150, 150)), 40);
    auto v = out.at<cv::Vec3b>(px({140, 140}).y, px({140, 140}).x + r_px);
    int best = 0;
    for (int dx = -2; dx <= 2; ++dx) {
        auto c = out.at<cv::Vec3b>(px({140, 140}).y, px({140, 140}).x + r_px + dx);
        best = std::max(best, static_cast<int>(c[2]) - static_cast<int>(c[0]));
    }
    (void)v;
    EXPECT_GT(best, 120);
    int drawn = 0, gaps = 0;
    for (int i = 0; i < 32; ++i) {
        const double a = 2 * CV_PI * i / 32;
        cv::Point p = px({150, 60}) + cv::Point(static_cast<int>(std::lround(r_px * std::cos(a))),
                                                static_cast<int>(std::lround(r_px * std::sin(a))));
        (nearest(out, p, cv::Scalar(220, 160, 0)) < 40 ? drawn : gaps)++;
    }
    EXPECT_GT(drawn, 4);
    EXPECT_GT(gaps, 4);
    EXPECT_LT(nearest(out, px({100, 150}), cv::Scalar(110, 110, 110)), 40);
}

// IP-FRM-05
TEST(LiveFrames, IP_FRM_05_HeaderLink) {
    GTEST_SKIP() << "header fields are set by the processing thread: see the process test IP_PRC_01";
}

// IP-FRM-06
TEST(LiveFrames, IP_FRM_06_Stop) {
    Rig r;
    Scene s;
    r.pipeline.start(s.target, 4.0);
    r.feed(render(s));
    ASSERT_TRUE(r.last_out);
    r.pipeline.stop();
    r.feed(render(s), 2);
    EXPECT_FALSE(r.last_out);
    auto status = r.pipeline.status();
    EXPECT_EQ(status.state, "not_calibrated");
    EXPECT_EQ(status.message, "live mode not active");
}

// IP-FRM-07
TEST(LiveFrames, IP_FRM_07_LensCorrected) {
    Scene s;
    s.distortion_k1 = -0.15;
    s.image = cv::Size(1280, 720);
    s.pose.fill = 0.9;
    s.holes_mm = {{15, 15}, {185, 20}, {20, 180}, {180, 185}};
    cv::Mat distorted = render(s);
    auto worst_error = [&](bool with_lens) {
        Rig r;
        if (with_lens) r.install_lens(-0.15, s.image);
        r.pipeline.start(s.target, 4.0);
        r.feed(distorted, 2);
        if (r.pipeline.status().state != "calibrated") return 1e9;
        double worst = 0;
        for (auto& truth : s.holes_mm) {
            double best = 1e9;
            for (auto& h : r.state.live->holes) best = std::min(best, dist_to_truth(h.paper_mm, truth));
            worst = std::max(worst, best);
        }
        return worst;
    };
    const double with_lens = worst_error(true);
    EXPECT_LT(with_lens, 0.6);
    EXPECT_GT(worst_error(false), with_lens);
}
