// doc/programs/image_processor.md#finding-the-paper
#include "paper_finder.hpp"

#include <algorithm>
#include <cmath>
#include <opencv2/imgproc.hpp>
#if __has_include(<opencv2/geometry.hpp>)
#include <opencv2/geometry.hpp>
#endif

#include "coordinates.hpp"

namespace image_processor {

using targetscope::model::Corners;
using targetscope::model::TargetType;

namespace {

constexpr double MIN_AREA_FRACTION = 0.10;
constexpr double ASPECT_TOLERANCE = 0.10;
constexpr double CENTER_TOLERANCE_MM = 3.0;
constexpr int WORK_SIDE_PX = 1280;
constexpr double CHECK_PX_PER_MM = 2.0;

double dist(const cv::Point2f& a, const cv::Point2f& b) { return std::hypot(a.x - b.x, a.y - b.y); }

// doc/programs/image_processor.md#finding-the-paper
double edge_ratio(const std::vector<cv::Point2f>& q) {
    const double w = 0.5 * (dist(q[0], q[1]) + dist(q[3], q[2]));
    const double h = 0.5 * (dist(q[0], q[3]) + dist(q[1], q[2]));
    return h > 0 ? w / h : 0;
}

// doc/programs/image_processor.md#finding-the-paper
double estimate_aspect(const std::vector<cv::Point2f>& q, cv::Size image) {
    const double cx = image.width / 2.0, cy = image.height / 2.0;
    auto m = [&](const cv::Point2f& p) { return cv::Vec3d(p.x - cx, p.y - cy, 1.0); };
    const cv::Vec3d m1 = m(q[0]), m2 = m(q[1]), m3 = m(q[3]), m4 = m(q[2]);
    const double d2 = (m2.cross(m4)).dot(m3);
    const double d3 = (m3.cross(m4)).dot(m2);
    if (std::abs(d2) < 1e-12 || std::abs(d3) < 1e-12) return edge_ratio(q);
    const double k2 = (m1.cross(m4)).dot(m3) / d2;
    const double k3 = (m1.cross(m4)).dot(m2) / d3;
    const cv::Vec3d n2 = k2 * m2 - m1, n3 = k3 * m3 - m1;
    double f2 = 0.9 * image.width;
    f2 *= f2;
    if (std::abs(n2[2] * n3[2]) > 1e-9) {
        const double est = -(n2[0] * n3[0] + n2[1] * n3[1]) / (n2[2] * n3[2]);
        if (est > 0 && std::sqrt(est) > 0.4 * image.width && std::sqrt(est) < 3.0 * image.width) f2 = est;
    }
    const double a = (n2[0] * n2[0] + n2[1] * n2[1]) / f2 + n2[2] * n2[2];
    const double b = (n3[0] * n3[0] + n3[1] * n3[1]) / f2 + n3[2] * n3[2];
    if (a <= 0 || b <= 0) return edge_ratio(q);
    return std::sqrt(a / b);
}

// doc/programs/image_processor.md#finding-the-paper
std::optional<cv::Point2d> black_center_mm(const cv::Mat& gray_paper, double px_per_mm) {
    cv::Mat blur, bin;
    cv::GaussianBlur(gray_paper, blur, cv::Size(5, 5), 0);
    cv::threshold(blur, bin, 0, 255, cv::THRESH_BINARY_INV | cv::THRESH_OTSU);
    const int margin = static_cast<int>(0.03 * std::max(bin.cols, bin.rows));
    bin(cv::Rect(0, 0, bin.cols, margin)).setTo(0);
    bin(cv::Rect(0, bin.rows - margin, bin.cols, margin)).setTo(0);
    bin(cv::Rect(0, 0, margin, bin.rows)).setTo(0);
    bin(cv::Rect(bin.cols - margin, 0, margin, bin.rows)).setTo(0);
    cv::Mat labels, stats, centroids;
    const int n = cv::connectedComponentsWithStats(bin, labels, stats, centroids, 8, CV_32S);
    int best = -1;
    int best_area = 0;
    for (int i = 1; i < n; ++i) {
        const int area = stats.at<int>(i, cv::CC_STAT_AREA);
        if (area > best_area) {
            best_area = area;
            best = i;
        }
    }
    if (best < 0 || best_area < 25 * px_per_mm * px_per_mm) return std::nullopt;
    cv::Mat mask = labels == best;
    std::vector<std::vector<cv::Point>> contours;
    cv::findContours(mask, contours, cv::RETR_EXTERNAL, cv::CHAIN_APPROX_NONE);
    if (contours.empty() || contours[0].size() < 5) return std::nullopt;
    auto biggest =
        std::max_element(contours.begin(), contours.end(), [](auto& a, auto& b) { return a.size() < b.size(); });
    cv::RotatedRect e = cv::fitEllipse(*biggest);
    const double ratio = std::max(e.size.width, e.size.height) / std::max(1.0f, std::min(e.size.width, e.size.height));
    if (ratio > 1.3) return std::nullopt;
    return cv::Point2d(e.center.x / px_per_mm, e.center.y / px_per_mm);
}

struct Quad {
    std::vector<cv::Point2f> corners;
    double area = 0;
};

// doc/programs/image_processor.md#finding-the-paper
std::vector<Quad> find_quads(const cv::Mat& gray_small) {
    cv::Mat blur, bin;
    cv::GaussianBlur(gray_small, blur, cv::Size(5, 5), 0);
    cv::threshold(blur, bin, 0, 255, cv::THRESH_BINARY | cv::THRESH_OTSU);
    std::vector<std::vector<cv::Point>> contours;
    cv::findContours(bin, contours, cv::RETR_EXTERNAL, cv::CHAIN_APPROX_SIMPLE);
    const double min_area = MIN_AREA_FRACTION * gray_small.total();
    std::vector<Quad> quads;
    for (auto& c : contours) {
        if (cv::contourArea(c) < min_area) continue;
        const double peri = cv::arcLength(c, true);
        for (double eps : {0.02, 0.03, 0.04}) {
            std::vector<cv::Point> approx;
            cv::approxPolyDP(c, approx, eps * peri, true);
            if (approx.size() != 4 || !cv::isContourConvex(approx)) continue;
            const double area = cv::contourArea(approx);
            if (area < min_area) continue;
            const int border = std::max(2, static_cast<int>(0.015 * std::max(gray_small.cols, gray_small.rows)));
            int at_border = 0;
            for (auto& p : approx)
                if (p.x <= border || p.y <= border || p.x >= gray_small.cols - 1 - border ||
                    p.y >= gray_small.rows - 1 - border)
                    ++at_border;
            if (at_border >= 3) continue;
            Quad q;
            q.area = area;
            for (auto& p : approx) q.corners.emplace_back(static_cast<float>(p.x), static_cast<float>(p.y));
            q.corners = order_corners(q.corners);
            quads.push_back(std::move(q));
            break;
        }
    }
    std::sort(quads.begin(), quads.end(), [](auto& a, auto& b) { return a.area > b.area; });
    return quads;
}

}

// doc/programs/image_processor.md#finding-the-paper
std::optional<Corners> find_paper(const cv::Mat& bgr, const TargetType* target_type) {
    if (bgr.empty()) return std::nullopt;
    cv::Mat gray;
    cv::cvtColor(bgr, gray, bgr.channels() == 1 ? cv::COLOR_GRAY2BGR : cv::COLOR_BGR2GRAY);
    if (bgr.channels() == 1) gray = bgr;
    const double scale = std::min(1.0, static_cast<double>(WORK_SIDE_PX) / std::max(gray.cols, gray.rows));
    cv::Mat small;
    if (scale < 1.0)
        cv::resize(gray, small, cv::Size(), scale, scale, cv::INTER_AREA);
    else
        small = gray;

    for (auto& quad : find_quads(small)) {
        std::vector<cv::Point2f> corners = quad.corners;
        for (auto& p : corners) p *= static_cast<float>(1.0 / scale);

        std::vector<cv::Point2f> refined = corners;
        cv::cornerSubPix(gray, refined, cv::Size(5, 5), cv::Size(-1, -1),
                         cv::TermCriteria(cv::TermCriteria::EPS | cv::TermCriteria::COUNT, 30, 0.01));
        for (size_t i = 0; i < 4; ++i)
            if (dist(refined[i], corners[i]) > 3.0 / scale) refined[i] = corners[i];
        corners = order_corners(refined);

        if (!target_type) return to_corners(corners);

        const double want = target_type->paper_width_mm / target_type->paper_height_mm;
        const double w_mm = target_type->paper_width_mm, h_mm = target_type->paper_height_mm;
        double best_err = 1e9;
        std::vector<cv::Point2f> best;
        for (int shift = 0; shift < 4; ++shift) {
            std::vector<cv::Point2f> q{corners[shift % 4], corners[(shift + 1) % 4], corners[(shift + 2) % 4],
                                       corners[(shift + 3) % 4]};
            const double aspect = estimate_aspect(q, gray.size());
            if (std::abs(aspect / want - 1.0) > ASPECT_TOLERANCE) continue;
            cv::Mat H = paper_homography(q, w_mm, h_mm, CHECK_PX_PER_MM);
            cv::Mat paper;
            cv::warpPerspective(
                gray, paper, H,
                cv::Size(static_cast<int>(w_mm * CHECK_PX_PER_MM), static_cast<int>(h_mm * CHECK_PX_PER_MM)),
                cv::INTER_LINEAR);
            auto center = black_center_mm(paper, CHECK_PX_PER_MM);
            if (!center) continue;
            const double err = std::hypot(center->x - target_type->center_x_mm, center->y - target_type->center_y_mm);
            if (err < best_err) {
                best_err = err;
                best = q;
            }
        }
        if (!best.empty() && best_err <= CENTER_TOLERANCE_MM) return to_corners(best);
    }
    return std::nullopt;
}

// doc/programs/image_processor.md#start-and-calibration
std::optional<Corners> find_paper_near(const cv::Mat& bgr, const Corners& known, const TargetType& t) {
    auto found = find_paper(bgr, &t);
    if (!found) return std::nullopt;
    auto a = to_points(known), b = to_points(*found);
    const double limit = 0.15 * cv::norm(a[0] - a[2]);
    for (int i = 0; i < 4; ++i)
        if (dist(a[i], b[i]) > limit) return std::nullopt;
    return found;
}

}
