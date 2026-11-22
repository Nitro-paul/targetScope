// doc/programs/image_processor.md#target-type-from-a-photo
#include "target_proposal.hpp"

#include <algorithm>
#include <cmath>
#include <numbers>
#include <opencv2/imgproc.hpp>

#include "cv_compat.hpp"

namespace image_processor {

namespace {

constexpr double MIN_CONTRAST = 60.0;
constexpr double MIN_BLACK_AREA_MM2 = 100.0;

}

// doc/programs/image_processor.md#target-type-from-a-photo
TargetProposal propose_rings(const cv::Mat& transformed_bgr, double px_per_mm) {
    TargetProposal p;
    cv::Mat gray;
    cv::cvtColor(transformed_bgr, gray, cv::COLOR_BGR2GRAY);
    cv::Mat blur;
    cv::GaussianBlur(gray, blur, cv::Size(5, 5), 0);

    double lo, hi;
    cv::minMaxLoc(blur, &lo, &hi);
    {
        std::vector<uchar> v(blur.begin<uchar>(), blur.end<uchar>());
        std::sort(v.begin(), v.end());
        lo = v[v.size() / 50];
        hi = v[v.size() * 49 / 50];
    }
    if (hi - lo < MIN_CONTRAST) return p;

    cv::Mat bin;
    cv::threshold(blur, bin, 0, 255, cv::THRESH_BINARY_INV | cv::THRESH_OTSU);
    const int margin = static_cast<int>(0.03 * std::max(bin.cols, bin.rows));
    bin(cv::Rect(0, 0, bin.cols, margin)).setTo(0);
    bin(cv::Rect(0, bin.rows - margin, bin.cols, margin)).setTo(0);
    bin(cv::Rect(0, 0, margin, bin.rows)).setTo(0);
    bin(cv::Rect(bin.cols - margin, 0, margin, bin.rows)).setTo(0);
    cv::Mat labels, stats, centroids;
    const int n = cv::connectedComponentsWithStats(bin, labels, stats, centroids, 8, CV_32S);
    int best = -1, best_area = 0;
    for (int i = 1; i < n; ++i) {
        const int a = stats.at<int>(i, cv::CC_STAT_AREA);
        if (a > best_area) {
            best_area = a;
            best = i;
        }
    }
    if (best < 0 || best_area < MIN_BLACK_AREA_MM2 * px_per_mm * px_per_mm) return p;
    cv::Mat mask = labels == best;
    std::vector<std::vector<cv::Point>> contours;
    cv::findContours(mask, contours, cv::RETR_EXTERNAL, cv::CHAIN_APPROX_NONE);
    auto biggest =
        std::max_element(contours.begin(), contours.end(), [](auto& a, auto& b) { return a.size() < b.size(); });
    if (biggest == contours.end() || biggest->size() < 5) return p;
    cv::RotatedRect e = cv::fitEllipse(*biggest);
    if (std::max(e.size.width, e.size.height) / std::max(1.0f, std::min(e.size.width, e.size.height)) > 1.3) return p;
    const cv::Point2d center(e.center.x, e.center.y);
    p.center_x_mm = center.x / px_per_mm;
    p.center_y_mm = center.y / px_per_mm;
    p.black_radius_mm = (e.size.width + e.size.height) / 4.0 / px_per_mm;

    {
        const double black_r_px = *p.black_radius_mm * px_per_mm;
        cv::Mat disc = cv::Mat::zeros(gray.size(), CV_8U);
        cv::circle(disc, cv::Point(static_cast<int>(center.x), static_cast<int>(center.y)),
                   static_cast<int>(black_r_px * 0.85), 255, cv::FILLED);
        cv::Mat light;
        cv::threshold(blur, light, 0, 255, cv::THRESH_BINARY | cv::THRESH_OTSU);
        cv::bitwise_and(light, disc, light);
        cv::Mat l2, st2, c2;
        const int m = cv::connectedComponentsWithStats(light, l2, st2, c2, 8, CV_32S);
        const int center_label = l2.at<int>(static_cast<int>(center.y), static_cast<int>(center.x));
        if (center_label > 0 && m > center_label) {
            const double area = st2.at<int>(center_label, cv::CC_STAT_AREA);
            const double r = std::sqrt(area / std::numbers::pi) / px_per_mm;
            if (r > 1.0 && r < *p.black_radius_mm * 0.9) p.inner_white_radius_mm = r;
        }
    }

    {
        const double edge = std::min({*p.center_x_mm, *p.center_y_mm, gray.cols / px_per_mm - *p.center_x_mm,
                                      gray.rows / px_per_mm - *p.center_y_mm});
        const double max_r_px = edge * px_per_mm;
        if (max_r_px > 20) {
            cv::Mat polar;
            const int radial_bins = static_cast<int>(max_r_px);
            cv::warpPolar(blur, polar, cv::Size(radial_bins, 720),
                          cv::Point2f(static_cast<float>(center.x), static_cast<float>(center.y)), max_r_px,
                          static_cast<int>(cv::WARP_POLAR_LINEAR) | static_cast<int>(cv::INTER_LINEAR));
            cv::Mat profile;
            cv::reduce(polar, profile, 0, cv::REDUCE_AVG, CV_32F);
            std::vector<float> v(radial_bins);
            for (int i = 0; i < radial_bins; ++i) v[i] = profile.at<float>(0, i);
            const int window = std::max(3, static_cast<int>(1.5 * px_per_mm));
            const int start = static_cast<int>((*p.black_radius_mm + 2.0) * px_per_mm);
            double outer = -1;
            for (int i = start; i < radial_bins - window; ++i) {
                float local = 0;
                int cnt = 0;
                for (int k = -2 * window; k <= 2 * window; ++k)
                    if (i + k >= 0 && i + k < radial_bins && std::abs(k) >= window) {
                        local += v[i + k];
                        ++cnt;
                    }
                local /= std::max(cnt, 1);
                bool is_min = true;
                for (int k = -window; k <= window; ++k)
                    if (v[i + k] < v[i]) is_min = false;
                if (is_min && local - v[i] > 8.0f) outer = i / px_per_mm;
            }
            if (outer > 0) p.outer_radius_mm = outer;
        }
    }
    return p;
}

}
