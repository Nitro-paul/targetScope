// doc/testing/image_processor.md#setup
#pragma once

#include <cmath>
#include <opencv2/calib3d.hpp>
#include <opencv2/imgproc.hpp>
#include <vector>

#include "coordinates.hpp"
#include "targetscope/model.hpp"
#include "targetscope/targets.hpp"

namespace image_processor::testing {

using targetscope::model::TargetType;

inline TargetType tt_ref() {
    TargetType t;
    t.name = "TT-REF";
    t.paper_width_mm = 200;
    t.paper_height_mm = 200;
    t.center_x_mm = 100;
    t.center_y_mm = 100;
    t.inner_white_radius_mm = 5;
    t.inner_white_score = 12;
    t.black_radius_mm = 30;
    t.black_score = 7;
    t.outer_radius_mm = 90;
    t.outer_score = 1;
    t.rings = targetscope::targets::generate_rings(t);
    return t;
}

struct Pose {
    double yaw_deg = 0;
    double pitch_deg = 0;
    double roll_deg = 0;
    double fill = 0.6;
    double dx_mm = 0, dy_mm = 0;
};

struct Scene {
    TargetType target = tt_ref();
    std::vector<cv::Point2d> holes_mm;
    double bullet_mm = 4.0;
    cv::Size image{1920, 1080};
    Pose pose;
    double shadow_fraction = 0;
    double hand_fraction = 0;
    bool paper = true;
    double noise = 2.0;
    double distortion_k1 = 0;
    double extra_spot_mm = 0;
    cv::Point2d extra_spot_pos{40, 40};
    bool blank = false;
    int background = 38;
    bool extra_line = false;
};

inline cv::Mat camera_matrix(cv::Size image) {
    cv::Mat K = cv::Mat::eye(3, 3, CV_64F);
    K.at<double>(0, 0) = K.at<double>(1, 1) = 1.5 * image.width * 0.78;
    K.at<double>(0, 2) = image.width / 2.0;
    K.at<double>(1, 2) = image.height / 2.0;
    return K;
}

inline std::vector<cv::Point2f> project_corners(const Scene& s) {
    const auto& t = s.target;
    cv::Mat K = camera_matrix(s.image);
    const double f = K.at<double>(0, 0);
    const double dist = f * std::max(t.paper_width_mm, t.paper_height_mm) / (s.pose.fill * s.image.height);
    const double ry = s.pose.yaw_deg * CV_PI / 180, rx = s.pose.pitch_deg * CV_PI / 180,
                 rz = s.pose.roll_deg * CV_PI / 180;
    cv::Mat Rx =
        cv::Mat(cv::Matx<double, 3, 3>(1, 0, 0, 0, std::cos(rx), -std::sin(rx), 0, std::sin(rx), std::cos(rx)));
    cv::Mat Ry =
        cv::Mat(cv::Matx<double, 3, 3>(std::cos(ry), 0, std::sin(ry), 0, 1, 0, -std::sin(ry), 0, std::cos(ry)));
    cv::Mat Rz =
        cv::Mat(cv::Matx<double, 3, 3>(std::cos(rz), -std::sin(rz), 0, std::sin(rz), std::cos(rz), 0, 0, 0, 1));
    cv::Mat R = Rz * Rx * Ry;
    cv::Mat rvec;
    cv::Rodrigues(R, rvec);
    cv::Mat tvec = cv::Mat(cv::Matx<double, 3, 1>(s.pose.dx_mm, s.pose.dy_mm, dist));
    const double w = t.paper_width_mm, h = t.paper_height_mm;
    std::vector<cv::Point3d> obj{{-w / 2, -h / 2, 0}, {w / 2, -h / 2, 0}, {w / 2, h / 2, 0}, {-w / 2, h / 2, 0}};
    std::vector<cv::Point2d> img;
    cv::projectPoints(obj, rvec, tvec, K, cv::Mat(), img);
    std::vector<cv::Point2f> out;
    for (auto& p : img) out.emplace_back(static_cast<float>(p.x), static_cast<float>(p.y));
    return out;
}

inline cv::Mat render_paper(const Scene& s, double ppm) {
    const auto& t = s.target;
    cv::Mat img(static_cast<int>(t.paper_height_mm * ppm), static_cast<int>(t.paper_width_mm * ppm), CV_8UC3,
                cv::Scalar(235, 235, 235));
    if (s.blank) return img;
    auto P = [&](double x, double y) {
        return cv::Point(static_cast<int>(std::lround(x * ppm)), static_cast<int>(std::lround(y * ppm)));
    };
    const cv::Point c = P(t.center_x_mm, t.center_y_mm);
    cv::circle(img, c, static_cast<int>(t.black_radius_mm * ppm), cv::Scalar(25, 25, 25), cv::FILLED, cv::LINE_AA);
    cv::circle(img, c, static_cast<int>(t.inner_white_radius_mm * ppm), cv::Scalar(235, 235, 235), cv::FILLED,
               cv::LINE_AA);
    for (auto& ring : t.rings) {
        const bool in_black = ring.radius_mm <= t.black_radius_mm + 0.01;
        cv::circle(img, c, static_cast<int>(std::lround(ring.radius_mm * ppm)),
                   in_black ? cv::Scalar(210, 210, 210) : cv::Scalar(70, 70, 70),
                   std::max(1, static_cast<int>(0.5 * ppm)), cv::LINE_AA);
    }
    const double r = s.bullet_mm / 2 * ppm;
    for (auto& h : s.holes_mm) {
        const int level = 100;
        cv::circle(img, P(h.x, h.y), static_cast<int>(std::lround(r)), cv::Scalar(level, level, level), cv::FILLED,
                   cv::LINE_AA);
    }
    if (s.extra_spot_mm > 0)
        cv::circle(img, P(s.extra_spot_pos.x, s.extra_spot_pos.y), static_cast<int>(s.extra_spot_mm / 2 * ppm),
                   cv::Scalar(100, 100, 100), cv::FILLED, cv::LINE_AA);
    if (s.extra_line) cv::rectangle(img, P(40, 150), P(52, 150.8), cv::Scalar(100, 100, 100), cv::FILLED);
    if (s.shadow_fraction > 0) img(cv::Rect(0, 0, img.cols, static_cast<int>(img.rows * s.shadow_fraction))) *= 0.45;
    return img;
}

inline cv::Mat distort_frame(const cv::Mat& ideal, const cv::Mat& K, double k1) {
    cv::Mat D = cv::Mat(cv::Matx<double, 5, 1>(k1, 0, 0, 0, 0));
    std::vector<cv::Point2f> pts, und;
    pts.reserve(ideal.total());
    for (int y = 0; y < ideal.rows; ++y)
        for (int x = 0; x < ideal.cols; ++x) pts.emplace_back(static_cast<float>(x), static_cast<float>(y));
    cv::undistortPoints(pts, und, K, D, cv::noArray(), K,
                        cv::TermCriteria(cv::TermCriteria::COUNT | cv::TermCriteria::EPS, 100, 1e-12));
    cv::Mat map(ideal.size(), CV_32FC2, und.data());
    cv::Mat out;
    cv::remap(ideal, out, map, cv::noArray(), cv::INTER_LINEAR, cv::BORDER_REPLICATE);
    return out;
}

inline cv::Mat render(const Scene& s) {
    cv::Mat frame(s.image, CV_8UC3, cv::Scalar(s.background, s.background + 2, s.background + 4));
    cv::Mat gradient(s.image, CV_8UC3);
    cv::randn(gradient, cv::Scalar(0, 0, 0), cv::Scalar(s.noise, s.noise, s.noise));
    if (s.paper) {
        const double ppm = 8.0;
        cv::Mat paper = render_paper(s, ppm);
        auto corners = project_corners(s);
        std::vector<cv::Point2f> src{{0, 0},
                                     {static_cast<float>(paper.cols), 0},
                                     {static_cast<float>(paper.cols), static_cast<float>(paper.rows)},
                                     {0, static_cast<float>(paper.rows)}};
        cv::Mat H = cv::getPerspectiveTransform(src, corners);
        cv::warpPerspective(paper, frame, H, s.image, cv::INTER_AREA | cv::WARP_INVERSE_MAP * 0,
                            cv::BORDER_TRANSPARENT);
        if (s.hand_fraction > 0) {
            std::vector<cv::Point2f> poly_src{
                {0, 0},
                {static_cast<float>(paper.cols), 0},
                {static_cast<float>(paper.cols), static_cast<float>(paper.rows * s.hand_fraction)},
                {0, static_cast<float>(paper.rows * s.hand_fraction)}};
            std::vector<cv::Point2f> poly_dst;
            cv::perspectiveTransform(poly_src, poly_dst, H);
            std::vector<cv::Point> poly;
            for (auto& p : poly_dst) poly.emplace_back(static_cast<int>(p.x), static_cast<int>(p.y));
            cv::fillConvexPoly(frame, poly, cv::Scalar(110, 140, 190));
        }
    }
    cv::add(frame, gradient, frame, cv::noArray(), CV_8UC3);
    if (s.distortion_k1 != 0) frame = distort_frame(frame, camera_matrix(s.image), s.distortion_k1);
    return frame;
}

}
