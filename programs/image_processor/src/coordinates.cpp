// doc/programs/image_processor.md#coordinate-systems
#include "coordinates.hpp"

#include <algorithm>
#include <opencv2/imgproc.hpp>

namespace image_processor {

using targetscope::model::Corners;
using targetscope::model::PixelPoint;

std::vector<cv::Point2f> to_points(const Corners& c) {
    return {{static_cast<float>(c.top_left.x), static_cast<float>(c.top_left.y)},
            {static_cast<float>(c.top_right.x), static_cast<float>(c.top_right.y)},
            {static_cast<float>(c.bottom_right.x), static_cast<float>(c.bottom_right.y)},
            {static_cast<float>(c.bottom_left.x), static_cast<float>(c.bottom_left.y)}};
}

Corners to_corners(const std::vector<cv::Point2f>& p) {
    auto pt = [](const cv::Point2f& q) { return PixelPoint{q.x, q.y}; };
    return Corners{pt(p[0]), pt(p[1]), pt(p[2]), pt(p[3])};
}

// doc/programs/image_processor.md#coordinate-systems
cv::Mat paper_homography(const std::vector<cv::Point2f>& corners, double paper_width_mm, double paper_height_mm,
                         double px_per_mm) {
    const double w = paper_width_mm * px_per_mm;
    const double h = paper_height_mm * px_per_mm;
    std::vector<cv::Point2f> dst{
        {0, 0}, {static_cast<float>(w), 0}, {static_cast<float>(w), static_cast<float>(h)}, {0, static_cast<float>(h)}};
    std::vector<cv::Point2d> s, d;
    for (auto& p : corners) s.emplace_back(p.x, p.y);
    for (auto& p : dst) d.emplace_back(p.x, p.y);
    cv::Mat A(8, 8, CV_64F, cv::Scalar(0)), b(8, 1, CV_64F);
    for (int i = 0; i < 4; ++i) {
        const double x = s[i].x, y = s[i].y, u = d[i].x, v = d[i].y;
        double r1[8] = {x, y, 1, 0, 0, 0, -u * x, -u * y};
        double r2[8] = {0, 0, 0, x, y, 1, -v * x, -v * y};
        for (int j = 0; j < 8; ++j) {
            A.at<double>(2 * i, j) = r1[j];
            A.at<double>(2 * i + 1, j) = r2[j];
        }
        b.at<double>(2 * i) = u;
        b.at<double>(2 * i + 1) = v;
    }
    cv::Mat x;
    cv::solve(A, b, x, cv::DECOMP_LU);
    cv::Mat H(3, 3, CV_64F);
    for (int i = 0; i < 8; ++i) H.at<double>(i / 3, i % 3) = x.at<double>(i);
    H.at<double>(2, 2) = 1.0;
    return H;
}

cv::Mat paper_homography(const Corners& corners, double paper_width_mm, double paper_height_mm, double px_per_mm) {
    return paper_homography(to_points(corners), paper_width_mm, paper_height_mm, px_per_mm);
}

// doc/programs/image_processor.md#coordinate-systems
targetscope::geometry::Polar paper_to_shot(const cv::Point2d& paper_mm, const targetscope::model::TargetType& t) {
    return targetscope::geometry::to_polar({paper_mm.x - t.center_x_mm, t.center_y_mm - paper_mm.y});
}

cv::Point2d shot_to_paper(const targetscope::geometry::Polar& shot, const targetscope::model::TargetType& t) {
    auto p = targetscope::geometry::to_point(shot);
    return {t.center_x_mm + p.x, t.center_y_mm - p.y};
}

// doc/programs/image_processor.md#finding-the-paper
std::vector<cv::Point2f> order_corners(const std::vector<cv::Point2f>& four) {
    std::vector<cv::Point2f> out(4);
    auto sum = [](const cv::Point2f& p) { return p.x + p.y; };
    auto diff = [](const cv::Point2f& p) { return p.y - p.x; };
    out[0] = *std::min_element(four.begin(), four.end(), [&](auto& a, auto& b) { return sum(a) < sum(b); });
    out[2] = *std::max_element(four.begin(), four.end(), [&](auto& a, auto& b) { return sum(a) < sum(b); });
    out[1] = *std::min_element(four.begin(), four.end(), [&](auto& a, auto& b) { return diff(a) < diff(b); });
    out[3] = *std::max_element(four.begin(), four.end(), [&](auto& a, auto& b) { return diff(a) < diff(b); });
    return out;
}

}
