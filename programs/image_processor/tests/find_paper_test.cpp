#include <gtest/gtest.h>

#include "coordinates.hpp"
#include "paper_finder.hpp"
#include "synthetic.hpp"

// IP-PAP
// doc/testing/image_processor.md#finding-the-paper-integration

using namespace image_processor;
using namespace image_processor::testing;

namespace {

double worst_error(const targetscope::model::Corners& found, const std::vector<cv::Point2f>& truth) {
    auto f = to_points(found);
    double worst = 0;
    for (int i = 0; i < 4; ++i) worst = std::max(worst, cv::norm(f[i] - truth[i]));
    return worst;
}

}

// IP-PAP-01
TEST(FindPaper, IP_PAP_01_Frontal) {
    Scene s;
    auto t = s.target;
    auto found = find_paper(render(s), &t);
    ASSERT_TRUE(found);
    EXPECT_LT(worst_error(*found, project_corners(s)), 1.0);
}

// IP-PAP-02
TEST(FindPaper, IP_PAP_02_AtAnAngle) {
    for (double yaw : {30.0, 45.0}) {
        Scene s;
        s.pose.yaw_deg = yaw;
        s.pose.pitch_deg = 10;
        auto t = s.target;
        auto found = find_paper(render(s), &t);
        ASSERT_TRUE(found) << yaw;
        EXPECT_LT(worst_error(*found, project_corners(s)), 2.0) << yaw;
    }
}

// IP-PAP-03
TEST(FindPaper, IP_PAP_03_PaperTooSmall) {
    Scene s;
    s.pose.fill = 0.6 * std::sqrt(0.08 / 0.30);
    auto t = s.target;
    EXPECT_FALSE(find_paper(render(s), &t));
}

// IP-PAP-04
TEST(FindPaper, IP_PAP_04_TwoPapers) {
    Scene big;
    Scene small;
    small.pose.fill = 0.25;
    cv::Mat a = render(big);
    cv::Mat b = render(small);
    cv::Mat tile;
    cv::resize(b, tile, cv::Size(), 0.3, 0.3);
    tile.copyTo(a(cv::Rect(5, 5, tile.cols, tile.rows)));
    auto t = big.target;
    auto found = find_paper(a, &t);
    ASSERT_TRUE(found);
    EXPECT_LT(worst_error(*found, project_corners(big)), 1.5);
}

// IP-PAP-05
TEST(FindPaper, IP_PAP_05_WrongAspectRatio) {
    Scene s;
    auto t = s.target;
    t.paper_height_mm = 140;
    t.center_y_mm = 70;
    EXPECT_FALSE(find_paper(render(s), &t));
}

// IP-PAP-06
TEST(FindPaper, IP_PAP_06_RotatedPaper) {
    for (double roll : {90.0, 180.0, 270.0}) {
        Scene s;
        s.target.center_x_mm = 80;
        s.target.center_y_mm = 120;
        s.pose.roll_deg = roll;
        auto t = s.target;
        auto found = find_paper(render(s), &t);
        ASSERT_TRUE(found) << roll;
        cv::Mat H = paper_homography(*found, 200, 200, 2.0);
        cv::Mat gray;
        cv::cvtColor(render(s), gray, cv::COLOR_BGR2GRAY);
        cv::Mat paper;
        cv::warpPerspective(gray, paper, H, cv::Size(400, 400));
        cv::Mat dark = paper < 90;
        cv::Moments m = cv::moments(dark, true);
        EXPECT_NEAR(m.m10 / m.m00 / 2.0, 80, 3.0) << roll;
        EXPECT_NEAR(m.m01 / m.m00 / 2.0, 120, 3.0) << roll;
    }
}

// IP-PAP-07
TEST(FindPaper, IP_PAP_07_NoPaper) {
    Scene s;
    s.paper = false;
    auto t = s.target;
    EXPECT_FALSE(find_paper(render(s), &t));
    EXPECT_FALSE(find_paper(render(s), nullptr));
}

// IP-PAP-08
TEST(FindPaper, IP_PAP_08_Photos) { GTEST_SKIP() << "no photos in testdata/photos/ yet (see testdata/README.md)"; }
