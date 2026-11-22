#include "lens_calibration.hpp"

#include <gtest/gtest.h>

#include <random>

#include "rig.hpp"
#include "synthetic.hpp"
#include "targetscope/messages.hpp"

// IP-LNS
// doc/testing/image_processor.md#lens-calibration-integration

using namespace image_processor;
using namespace image_processor::testing;

namespace {

constexpr double TRUE_K1 = -0.22;
const cv::Size IMAGE(1280, 720);

cv::Mat checkerboard_view(int index, double k1 = TRUE_K1, bool with_board = true) {
    cv::Mat frame(IMAGE, CV_8UC3, cv::Scalar(70, 70, 70));
    if (with_board) {
        cv::Mat board(280, 400, CV_8UC3, cv::Scalar(255, 255, 255));
        for (int r = 0; r < 7; ++r)
            for (int c = 0; c < 10; ++c)
                if ((r + c) % 2 == 0) board(cv::Rect(c * 40, r * 40, 40, 40)).setTo(cv::Scalar(0, 0, 0));
        cv::copyMakeBorder(board, board, 40, 40, 40, 40, cv::BORDER_CONSTANT, cv::Scalar(255, 255, 255));
        std::mt19937 rng(index * 7919 + 13);
        std::uniform_real_distribution<double> u(-1, 1);
        cv::Mat K = camera_matrix(IMAGE);
        const double yaw = u(rng) * 38 * CV_PI / 180, pitch = u(rng) * 32 * CV_PI / 180,
                     roll = u(rng) * 20 * CV_PI / 180;
        cv::Mat Rx = cv::Mat(
            cv::Matx<double, 3, 3>(1, 0, 0, 0, std::cos(pitch), -std::sin(pitch), 0, std::sin(pitch), std::cos(pitch)));
        cv::Mat Ry =
            cv::Mat(cv::Matx<double, 3, 3>(std::cos(yaw), 0, std::sin(yaw), 0, 1, 0, -std::sin(yaw), 0, std::cos(yaw)));
        cv::Mat Rz = cv::Mat(
            cv::Matx<double, 3, 3>(std::cos(roll), -std::sin(roll), 0, std::sin(roll), std::cos(roll), 0, 0, 0, 1));
        cv::Mat R = Rz * Rx * Ry, rvec;
        cv::Rodrigues(R, rvec);
        const double dist = 330 + 120 * u(rng);
        cv::Mat tvec = cv::Mat(cv::Matx<double, 3, 1>(0.30 * dist * 0.30 * u(rng), 0.24 * dist * 0.25 * u(rng), dist));
        std::vector<cv::Point3d> obj{{-72, -54, 0}, {72, -54, 0}, {72, 54, 0}, {-72, 54, 0}};
        std::vector<cv::Point2d> img;
        cv::projectPoints(obj, rvec, tvec, K, cv::Mat(), img);
        std::vector<cv::Point2f> dst, src{{0, 0},
                                          {static_cast<float>(board.cols), 0},
                                          {static_cast<float>(board.cols), static_cast<float>(board.rows)},
                                          {0, static_cast<float>(board.rows)}};
        for (auto& p : img) dst.emplace_back(static_cast<float>(p.x), static_cast<float>(p.y));
        cv::warpPerspective(board, frame, cv::getPerspectiveTransform(src, dst), IMAGE, cv::INTER_LINEAR,
                            cv::BORDER_TRANSPARENT);
    }
    return distort_frame(frame, camera_matrix(IMAGE), k1);
}

LensCalibrationSession session_with(int views) {
    LensCalibrationSession s;
    s.camera.camera_key = "cam";
    s.camera.width_px = IMAGE.width;
    s.camera.height_px = IMAGE.height;
    for (int i = 0; i < views; ++i) EXPECT_TRUE(capture_view(s, checkerboard_view(i)));
    return s;
}

}

// IP-LNS-01
TEST(LensCalibration, IP_LNS_01_DuringLiveMode) {
    GTEST_SKIP() << "decided by the request handler: see the process test IP_PRC_06";
}

// IP-LNS-02
TEST(LensCalibration, IP_LNS_02_Start) {
    GTEST_SKIP() << "decided by the request handler: see the process test IP_PRC_06";
}

// IP-LNS-03
TEST(LensCalibration, IP_LNS_03_Capture) {
    LensCalibrationSession s;
    EXPECT_TRUE(capture_view(s, checkerboard_view(1)));
    EXPECT_EQ(s.views.size(), 1u);
    EXPECT_FALSE(capture_view(s, checkerboard_view(2, TRUE_K1, false)));
    EXPECT_EQ(s.views.size(), 1u);
}

// IP-LNS-04
TEST(LensCalibration, IP_LNS_04_TooFewViews) {
    auto s = session_with(9);
    try {
        finish(s);
        FAIL();
    } catch (const targetscope::ServiceError& e) {
        EXPECT_EQ(e.code, targetscope::ErrorCode::ValidationFailed);
    }
    EXPECT_EQ(s.views.size(), 9u);
}

// IP-LNS-05
TEST(LensCalibration, IP_LNS_05_Compute) {
    auto s = session_with(12);
    LensCalibration c = finish(s);
    EXPECT_LT(c.rms_px, 0.5);
    EXPECT_NEAR(c.dist_coeffs.at<double>(0), TRUE_K1, 0.05 * std::abs(TRUE_K1));
    EXPECT_NEAR(c.camera_matrix.at<double>(0, 0), camera_matrix(IMAGE).at<double>(0, 0),
                0.05 * camera_matrix(IMAGE).at<double>(0, 0));
    auto entry = to_setting(c);
    EXPECT_EQ(entry["width_px"], IMAGE.width);
    EXPECT_EQ(entry["camera_matrix"].size(), 9u);
    EXPECT_EQ(entry["dist_coeffs"].size(), 5u);
    nlohmann::json all{{"cam", entry}};
    CameraInfo cam;
    cam.camera_key = "cam";
    auto back = from_setting(all, cam);
    ASSERT_TRUE(back);
    EXPECT_NEAR(back->rms_px, c.rms_px, 1e-12);
    EXPECT_NEAR(back->dist_coeffs.at<double>(0), c.dist_coeffs.at<double>(0), 1e-12);
    cam.camera_key = "other";
    EXPECT_FALSE(from_setting(all, cam));
}

// IP-LNS-06
TEST(LensCalibration, IP_LNS_06_Applied) {
    auto s = session_with(12);
    LensCalibration c = finish(s);
    LensCorrector lens(c, IMAGE);
    cv::Mat grid(IMAGE, CV_8UC3, cv::Scalar(255, 255, 255));
    for (int x = 160; x < IMAGE.width; x += 160) cv::line(grid, {x, 0}, {x, IMAGE.height - 1}, cv::Scalar(0, 0, 0), 2);
    for (int y = 120; y < IMAGE.height; y += 120) cv::line(grid, {0, y}, {IMAGE.width - 1, y}, cv::Scalar(0, 0, 0), 2);
    cv::Mat distorted = distort_frame(grid, camera_matrix(IMAGE), TRUE_K1);
    cv::Mat fixed;
    lens.correct(distorted, fixed);
    auto line_x = [&](const cv::Mat& img, int y) {
        cv::Mat g;
        cv::cvtColor(img, g, cv::COLOR_BGR2GRAY);
        double sum = 0, weight = 0;
        for (int x = 100; x < 230; ++x) {
            const double w = 255 - g.at<uchar>(y, x);
            sum += w * x;
            weight += w;
        }
        return weight > 0 ? sum / weight : -1;
    };
    double worst = 0, worst_before = 0;
    const double ref = line_x(fixed, 300), ref_before = line_x(distorted, 300);
    for (int y = 150; y < 600; y += 30) {
        if (y % 120 == 0) continue;
        worst = std::max(worst, std::abs(line_x(fixed, y) - ref));
        worst_before = std::max(worst_before, std::abs(line_x(distorted, y) - ref_before));
    }
    EXPECT_GT(worst_before, 1.5);
    EXPECT_LT(worst, 0.7);
}

// IP-LNS-07
TEST(LensCalibration, IP_LNS_07_CameraChangesWhileCollecting) {
    GTEST_SKIP() << "decided by the request handler: see the process test IP_PRC_06";
}

// IP-LNS-08
TEST(LensCalibration, IP_LNS_08_Cancel) {
    GTEST_SKIP() << "decided by the request handler: see the process test IP_PRC_06";
}

// IP-LNS-09
TEST(LensCalibration, IP_LNS_09_Delete) {
    GTEST_SKIP() << "decided by the request handler: see the process test IP_PRC_06";
}

// IP-LNS-10
TEST(LensCalibration, IP_LNS_10_OtherResolution) {
    GTEST_SKIP() << "decided by the request handler: see the process test IP_PRC_06";
}

// IP-LNS-11
TEST(LensCalibration, IP_LNS_11_OtherCamera) {
    GTEST_SKIP() << "decided by the request handler: see the process test IP_PRC_06";
}

// IP-LNS-12
TEST(LensCalibration, IP_LNS_12_RawFeedStaysUncorrected) {
    GTEST_SKIP() << "the raw feed is published by camera_reader, which never reads a lens calibration";
}

// IP-LNS-13
TEST(LensCalibration, IP_LNS_13_AccuracyNearTheEdge) {
    Scene s;
    s.image = IMAGE;
    s.distortion_k1 = TRUE_K1;
    s.pose.fill = 0.9;
    s.holes_mm = {{15, 15}, {185, 20}, {20, 180}, {180, 185}};
    cv::Mat frame = render(s);
    LensCalibration c = finish(session_with(14));
    c.camera_key = "cam";
    Rig r;
    r.state.lens = c;
    r.state.lens_size = IMAGE;
    r.state.corrector = std::make_shared<LensCorrector>(c, IMAGE);
    r.pipeline.start(s.target, 4.0);
    r.feed(frame, 2);
    ASSERT_EQ(r.pipeline.status().state, "calibrated");
    ASSERT_EQ(r.state.live->holes.size(), 4u);
    for (auto& truth : s.holes_mm) {
        double best = 1e9;
        for (auto& h : r.state.live->holes) best = std::min(best, dist_to_truth(h.paper_mm, truth));
        EXPECT_LT(best, 1.0);
    }
}
