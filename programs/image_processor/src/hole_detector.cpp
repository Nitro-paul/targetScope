// doc/programs/image_processor.md#new-holes
#include "hole_detector.hpp"

#include <algorithm>
#include <cmath>
#include <cstdio>
#include <functional>
#include <numbers>
#include <opencv2/imgproc.hpp>

#include "coordinates.hpp"
#include "cv_compat.hpp"

namespace image_processor {

namespace {

constexpr double MIN_DIAMETER = 0.6;
constexpr double MAX_DIAMETER = 2.0;
constexpr double MIN_ROUNDNESS = 0.6;
constexpr double MIN_NARROW_SIDE = 0.5;
constexpr double OCCLUSION_FRACTION = 0.05;
constexpr int OCCLUSION_LEVEL = 40;
constexpr int NEW_HOLE_LEVEL = 35;
constexpr int SINGLE_IMAGE_LEVEL = 25;
constexpr double OVERLAP_AREA_FACTOR = 1.6;
constexpr double WARN_MAX_DIAMETER = 8.0;
constexpr double EDGE_MARGIN_MM = 2.0;
constexpr double WORK_HOLE_PX = 12.0;

int odd(double v) {
    int k = static_cast<int>(std::lround(v));
    if (k % 2 == 0) ++k;
    return std::max(k, 3);
}

double hole_area(double bullet_mm) { return std::numbers::pi * bullet_mm * bullet_mm / 4.0; }

// doc/programs/image_processor.md#new-holes
std::vector<Blob> blobs_of(const cv::Mat& mask, double px_per_mm, double bullet_mm, std::vector<std::string>* warnings,
                           bool keep_all_sizes = false, const std::function<bool(const Blob&)>& skip = {}) {
    std::vector<Blob> out;
    cv::Mat labels, stats, centroids;
    const int n = cv::connectedComponentsWithStats(mask, labels, stats, centroids, 8, CV_32S);
    for (int i = 1; i < n; ++i) {
        const int area_px = stats.at<int>(i, cv::CC_STAT_AREA);
        if (area_px < 6) continue;
        const cv::Rect box(stats.at<int>(i, cv::CC_STAT_LEFT), stats.at<int>(i, cv::CC_STAT_TOP),
                           stats.at<int>(i, cv::CC_STAT_WIDTH), stats.at<int>(i, cv::CC_STAT_HEIGHT));
        cv::Mat roi = (labels(box) == i);
        std::vector<std::vector<cv::Point>> contours;
        cv::findContours(roi, contours, cv::RETR_EXTERNAL, cv::CHAIN_APPROX_NONE);
        if (contours.empty()) continue;
        auto biggest =
            std::max_element(contours.begin(), contours.end(), [](auto& a, auto& b) { return a.size() < b.size(); });
        std::vector<cv::Point> hull;
        cv::convexHull(*biggest, hull);
        const double hull_perimeter = cv::arcLength(hull, true);
        Blob b;
        b.area_mm2 = area_px / (px_per_mm * px_per_mm);
        b.diameter_mm = std::sqrt(4.0 * b.area_mm2 / std::numbers::pi);
        b.roundness = hull_perimeter > 0
                          ? std::min(1.0, 4.0 * std::numbers::pi * area_px / (hull_perimeter * hull_perimeter))
                          : 0;
        b.paper_mm = {centroids.at<double>(i, 0) / px_per_mm, centroids.at<double>(i, 1) / px_per_mm};
        b.major_mm = b.diameter_mm;
        b.minor_mm = b.diameter_mm;
        if (biggest->size() >= 5) {
            cv::RotatedRect e = cv::fitEllipse(*biggest);
            b.major_mm = std::max(e.size.width, e.size.height) / px_per_mm;
            b.minor_mm = std::min(e.size.width, e.size.height) / px_per_mm;
            b.orientation_rad = (e.size.width >= e.size.height ? e.angle : e.angle + 90.0) * std::numbers::pi / 180.0;
        }
        const double ratio = b.diameter_mm / bullet_mm;
        if (skip && skip(b)) continue;
        if (b.roundness < MIN_ROUNDNESS || b.minor_mm < MIN_NARROW_SIDE * bullet_mm) continue;
        if (ratio < MIN_DIAMETER || ratio > MAX_DIAMETER) {
            const bool merged_pair = b.area_mm2 > OVERLAP_AREA_FACTOR * hole_area(bullet_mm) && ratio <= 2.0;
            if (!merged_pair && warnings && ratio > MAX_DIAMETER && ratio <= WARN_MAX_DIAMETER) {
                char text[160];
                std::snprintf(text, sizeof text, "a spot of %.1f mm at (%.0f, %.0f) mm is larger than a bullet hole",
                              b.diameter_mm, b.paper_mm.x, b.paper_mm.y);
                warnings->push_back(text);
            }
            if (!keep_all_sizes) continue;
        }
        out.push_back(b);
    }
    return out;
}

}

// doc/programs/image_processor.md#new-holes
bool is_occluded(const cv::Mat& gray, const cv::Mat& reference) {
    cv::Mat diff;
    cv::absdiff(gray, reference, diff);
    const int strong = cv::countNonZero(diff > OCCLUSION_LEVEL);
    return strong > OCCLUSION_FRACTION * static_cast<double>(diff.total());
}

// doc/programs/image_processor.md#new-holes
std::vector<Blob> find_new_blobs(const cv::Mat& gray, const cv::Mat& reference, double px_per_mm,
                                 double bullet_diameter_mm, const std::vector<cv::Point2d>& known_holes_mm) {
    cv::Mat diff, mask;
    cv::absdiff(gray, reference, diff);
    cv::GaussianBlur(diff, diff, cv::Size(3, 3), 0);
    cv::threshold(diff, mask, NEW_HOLE_LEVEL, 255, cv::THRESH_BINARY);
    const int k = odd(0.2 * bullet_diameter_mm * px_per_mm);
    cv::morphologyEx(mask, mask, cv::MORPH_OPEN, cv::getStructuringElement(cv::MORPH_ELLIPSE, cv::Size(k, k)));
    const int kc = odd(0.4 * bullet_diameter_mm * px_per_mm);
    cv::morphologyEx(mask, mask, cv::MORPH_CLOSE, cv::getStructuringElement(cv::MORPH_ELLIPSE, cv::Size(kc, kc)));
    std::vector<Blob> out = blobs_of(mask, px_per_mm, bullet_diameter_mm, nullptr);

    const double single_px = hole_area(bullet_diameter_mm) * px_per_mm * px_per_mm;
    for (const auto& known : known_holes_mm) {
        const cv::Point c(static_cast<int>(known.x * px_per_mm), static_cast<int>(known.y * px_per_mm));
        const int r_in = static_cast<int>(0.5 * bullet_diameter_mm * px_per_mm);
        const int r_out = static_cast<int>(1.4 * bullet_diameter_mm * px_per_mm);
        cv::Rect box(c.x - r_out, c.y - r_out, 2 * r_out + 1, 2 * r_out + 1);
        box &= cv::Rect(0, 0, mask.cols, mask.rows);
        if (box.area() == 0) continue;
        cv::Mat ring(box.size(), CV_8U, cv::Scalar(0));
        cv::circle(ring, c - box.tl(), r_out, 255, cv::FILLED);
        cv::circle(ring, c - box.tl(), r_in, 0, cv::FILLED);
        cv::Mat touching;
        cv::bitwise_and(mask(box), ring, touching);
        if (cv::countNonZero(touching) < 0.2 * single_px) continue;
        cv::Mat joined = mask(box).clone();
        cv::circle(joined, c - box.tl(), static_cast<int>(0.5 * bullet_diameter_mm * px_per_mm), 255, cv::FILLED);
        auto unions = blobs_of(joined, px_per_mm, bullet_diameter_mm, nullptr, true);
        for (auto& u : unions) {
            if (u.area_mm2 <= OVERLAP_AREA_FACTOR * hole_area(bullet_diameter_mm) ||
                u.area_mm2 > 2.4 * hole_area(bullet_diameter_mm))
                continue;
            u.paper_mm += cv::Point2d(box.x / px_per_mm, box.y / px_per_mm);
            auto pair = split_overlapping(u, bullet_diameter_mm);
            if (pair.size() != 2) continue;
            auto d = [&](const Blob& b) { return std::hypot(b.paper_mm.x - known.x, b.paper_mm.y - known.y); };
            Blob fresh = d(pair[0]) > d(pair[1]) ? pair[0] : pair[1];
            out.erase(std::remove_if(out.begin(), out.end(),
                                     [&](const Blob& o) {
                                         return std::hypot(o.paper_mm.x - fresh.paper_mm.x,
                                                           o.paper_mm.y - fresh.paper_mm.y) < bullet_diameter_mm;
                                     }),
                      out.end());
            out.push_back(fresh);
        }
    }
    return out;
}

// doc/programs/image_processor.md#new-holes
std::vector<Blob> update_candidates(std::vector<Candidate>& candidates, const std::vector<Blob>& blobs,
                                    double bullet_diameter_mm, int confirm_frames) {
    std::vector<bool> matched(candidates.size(), false);
    std::vector<Blob> confirmed;
    std::vector<Candidate> fresh;
    for (const auto& b : blobs) {
        int best = -1;
        double best_d = bullet_diameter_mm / 2.0;
        for (size_t i = 0; i < candidates.size(); ++i) {
            if (matched[i]) continue;
            const double d =
                std::hypot(candidates[i].paper_mm.x - b.paper_mm.x, candidates[i].paper_mm.y - b.paper_mm.y);
            if (d <= best_d) {
                best_d = d;
                best = static_cast<int>(i);
            }
        }
        if (best >= 0) {
            matched[best] = true;
            auto& c = candidates[best];
            c.paper_mm = b.paper_mm;
            c.diameter_mm = b.diameter_mm;
            c.roundness = b.roundness;
            c.merged = b.merged;
            c.major_mm = b.major_mm;
            c.orientation_rad = b.orientation_rad;
            c.area_mm2 = b.area_mm2;
            ++c.seen_frames;
            c.missed_frames = 0;
        } else {
            Candidate c;
            c.paper_mm = b.paper_mm;
            c.diameter_mm = b.diameter_mm;
            c.roundness = b.roundness;
            c.merged = b.merged;
            c.major_mm = b.major_mm;
            c.orientation_rad = b.orientation_rad;
            c.area_mm2 = b.area_mm2;
            c.seen_frames = 1;
            fresh.push_back(c);
        }
    }
    for (size_t i = 0; i < candidates.size(); ++i)
        if (!matched[i]) {
            candidates[i].seen_frames = 0;
            ++candidates[i].missed_frames;
        }
    candidates.erase(std::remove_if(candidates.begin(), candidates.end(), [](auto& c) { return c.missed_frames >= 3; }),
                     candidates.end());
    for (auto& c : fresh) candidates.push_back(c);
    for (auto it = candidates.begin(); it != candidates.end();) {
        if (it->seen_frames >= confirm_frames) {
            Blob b;
            b.paper_mm = it->paper_mm;
            b.diameter_mm = it->diameter_mm;
            b.roundness = it->roundness;
            b.merged = it->merged;
            b.major_mm = it->major_mm;
            b.orientation_rad = it->orientation_rad;
            b.area_mm2 = it->area_mm2;
            confirmed.push_back(b);
            it = candidates.erase(it);
        } else {
            ++it;
        }
    }
    return confirmed;
}

// doc/programs/image_processor.md#new-holes
std::vector<Blob> split_overlapping(const Blob& blob, double bullet_diameter_mm) {
    if (blob.area_mm2 <= OVERLAP_AREA_FACTOR * hole_area(bullet_diameter_mm)) return {blob};
    const double separation = std::max(0.0, blob.major_mm - bullet_diameter_mm);
    const double dx = std::cos(blob.orientation_rad) * separation / 2.0;
    const double dy = std::sin(blob.orientation_rad) * separation / 2.0;
    Blob a = blob, b = blob;
    a.paper_mm = {blob.paper_mm.x - dx, blob.paper_mm.y - dy};
    b.paper_mm = {blob.paper_mm.x + dx, blob.paper_mm.y + dy};
    a.diameter_mm = b.diameter_mm = bullet_diameter_mm;
    a.area_mm2 = b.area_mm2 = hole_area(bullet_diameter_mm);
    a.merged = b.merged = true;
    return {a, b};
}

// doc/programs/image_processor.md#new-holes
float confidence(const Blob& blob, double bullet_diameter_mm) {
    if (blob.merged) return 0.4f;
    const double size_score = 1.0 - std::min(1.0, std::abs(blob.diameter_mm / bullet_diameter_mm - 1.0));
    const double c = 0.4 + 0.6 * (0.5 * size_score + 0.5 * std::min(1.0, blob.roundness / 0.9));
    return static_cast<float>(std::clamp(c, 0.0, 1.0));
}

// doc/programs/image_processor.md#holes-in-a-single-image
DetectionResult detect_in_single_image(const cv::Mat& transformed_bgr, double px_per_mm,
                                       const targetscope::model::TargetType& t, double bullet_diameter_mm) {
    DetectionResult result;
    cv::Mat gray;
    cv::cvtColor(transformed_bgr, gray, transformed_bgr.channels() == 1 ? cv::COLOR_GRAY2BGR : cv::COLOR_BGR2GRAY);
    if (transformed_bgr.channels() == 1) gray = transformed_bgr;

    const double scale = std::min(1.0, WORK_HOLE_PX / (bullet_diameter_mm * px_per_mm));
    cv::Mat small;
    if (scale < 1.0)
        cv::resize(gray, small, cv::Size(), scale, scale, cv::INTER_AREA);
    else
        small = gray;
    const double ppm = px_per_mm * scale;

    const int se = odd(4.0 * bullet_diameter_mm * ppm);
    const cv::Mat element = cv::getStructuringElement(cv::MORPH_ELLIPSE, cv::Size(se, se));
    cv::Mat dark, light, deviation, mask;
    cv::morphologyEx(small, dark, cv::MORPH_BLACKHAT, element);
    cv::morphologyEx(small, light, cv::MORPH_TOPHAT, element);
    cv::Mat black_ring = cv::Mat::zeros(small.size(), CV_8U);
    const cv::Point center(static_cast<int>(std::lround(t.center_x_mm * ppm)),
                           static_cast<int>(std::lround(t.center_y_mm * ppm)));
    cv::circle(black_ring, center, static_cast<int>(std::lround(t.black_radius_mm * ppm)), 255, cv::FILLED);
    cv::circle(black_ring, center, static_cast<int>(std::lround(t.inner_white_radius_mm * ppm)), 0, cv::FILLED);
    deviation = dark.clone();
    light.copyTo(deviation, black_ring);
    cv::threshold(deviation, mask, SINGLE_IMAGE_LEVEL, 255, cv::THRESH_BINARY);
    const int k = odd(0.3 * bullet_diameter_mm * ppm);
    cv::morphologyEx(mask, mask, cv::MORPH_OPEN, cv::getStructuringElement(cv::MORPH_ELLIPSE, cv::Size(k, k)));
    const int margin = static_cast<int>(EDGE_MARGIN_MM * ppm);
    mask(cv::Rect(0, 0, mask.cols, margin)).setTo(0);
    mask(cv::Rect(0, mask.rows - margin, mask.cols, margin)).setTo(0);
    mask(cv::Rect(0, 0, margin, mask.rows)).setTo(0);
    mask(cv::Rect(mask.cols - margin, 0, margin, mask.rows)).setTo(0);

    auto is_print = [&](const Blob& b) {
        return std::hypot(b.paper_mm.x - t.center_x_mm, b.paper_mm.y - t.center_y_mm) < t.inner_white_radius_mm &&
               b.area_mm2 > 2.2 * hole_area(bullet_diameter_mm);
    };
    for (auto& b : blobs_of(mask, ppm, bullet_diameter_mm, &result.warnings, true, is_print)) {
        const double ratio = b.diameter_mm / bullet_diameter_mm;
        if (ratio >= MIN_DIAMETER && ratio <= MAX_DIAMETER) {
            result.holes.push_back(b);
        } else if (b.area_mm2 > OVERLAP_AREA_FACTOR * hole_area(bullet_diameter_mm) && ratio <= 2.5) {
            for (auto& h : split_overlapping(b, bullet_diameter_mm)) result.holes.push_back(h);
        }
    }
    return result;
}

}
