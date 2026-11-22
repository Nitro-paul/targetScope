#include "coordinates.hpp"

#include <gtest/gtest.h>

#include <algorithm>
#include <numeric>

#include "lens_calibration.hpp"
#include "synthetic.hpp"

// IP-GEO
// doc/testing/image_processor.md#coordinates-unit

using namespace image_processor;
using namespace image_processor::testing;

// IP-GEO-01
TEST(Coordinates, IP_GEO_01_HomographyFromCorners) {
    std::vector<cv::Point2f> corners{{410, 95}, {1490, 120}, {1460, 1010}, {430, 985}};
    cv::Mat H = paper_homography(corners, 200, 200, 6);
    std::vector<cv::Point2f> out;
    cv::perspectiveTransform(corners, out, H);
    const std::vector<cv::Point2f> want{{0, 0}, {1200, 0}, {1200, 1200}, {0, 1200}};
    for (int i = 0; i < 4; ++i) EXPECT_LT(cv::norm(out[i] - want[i]), 1e-3);
}

// IP-GEO-02
TEST(Coordinates, IP_GEO_02_PaperToShot) {
    auto t = tt_ref();
    struct Case {
        cv::Point2d paper;
        double angle, radius;
    };
    for (auto c :
         {Case{{110, 100}, 0.25, 10}, Case{{100, 90}, 0, 10}, Case{{100, 100}, 0, 0}, Case{{90, 100}, 0.75, 10}}) {
        auto s = paper_to_shot(c.paper, t);
        EXPECT_NEAR(s.angle_turns, c.angle, 1e-9);
        EXPECT_NEAR(s.radius_mm, c.radius, 1e-9);
        auto p = shot_to_paper(s, t);
        EXPECT_NEAR(p.x, c.paper.x, 1e-9);
        EXPECT_NEAR(p.y, c.paper.y, 1e-9);
    }
}

// IP-GEO-03
TEST(Coordinates, IP_GEO_03_CornerOrder) {
    const std::vector<cv::Point2f> truth{{410, 95}, {1490, 120}, {1460, 1010}, {430, 985}};
    std::vector<int> idx{0, 1, 2, 3};
    int n = 0;
    do {
        std::vector<cv::Point2f> shuffled;
        for (int i : idx) shuffled.push_back(truth[i]);
        EXPECT_EQ(order_corners(shuffled), truth);
        ++n;
    } while (std::next_permutation(idx.begin(), idx.end()));
    EXPECT_EQ(n, 24);
}

// IP-GEO-04
TEST(Coordinates, IP_GEO_04_CombinedMap) {
    LensCalibration c;
    c.width_px = 640;
    c.height_px = 480;
    c.camera_matrix = camera_matrix(cv::Size(640, 480));
    c.dist_coeffs = cv::Mat(cv::Matx<double, 5, 1>(-0.25, 0.05, 0, 0, 0));
    LensCorrector lens(c, cv::Size(640, 480));
    cv::Mat raw(480, 640, CV_8UC3);
    cv::randu(raw, 0, 255);
    cv::GaussianBlur(raw, raw, cv::Size(9, 9), 0);
    std::vector<cv::Point2f> corners{{100, 80}, {540, 90}, {530, 400}, {110, 390}};
    cv::Mat H = paper_homography(corners, 100, 70, 4);
    cv::Mat one;
    cv::remap(raw, one, lens.combined_map(H, cv::Size(400, 280)), cv::noArray(), cv::INTER_LINEAR);
    cv::Mat corrected, two;
    lens.correct(raw, corrected);
    cv::warpPerspective(corrected, two, H, cv::Size(400, 280), cv::INTER_LINEAR);
    cv::Mat diff;
    cv::absdiff(one, two, diff);
    cv::Mat inner = diff(cv::Rect(20, 20, 360, 240));
    double mean = cv::mean(inner)[0];
    EXPECT_LT(mean, 1.0);
    std::vector<cv::Point2f> pts{{100, 100}, {320, 240}, {600, 400}};
    auto back = lens.to_raw(lens.to_corrected(pts));
    for (size_t i = 0; i < pts.size(); ++i) EXPECT_LT(cv::norm(pts[i] - back[i]), 0.1);
}
