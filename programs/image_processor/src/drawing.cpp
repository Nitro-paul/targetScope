// doc/frame_transport.md#live-frames
#include "drawing.hpp"

#include <cmath>
#include <opencv2/imgproc.hpp>

namespace image_processor {

namespace {

const cv::Scalar GRAY(150, 150, 150);
const cv::Scalar COUNTED(0, 140, 255);
const cv::Scalar NEWEST(0, 0, 255);
const cv::Scalar CANDIDATE(220, 160, 0);
const cv::Scalar RING(110, 110, 110);

void dashed_circle(cv::Mat& img, cv::Point center, int radius, const cv::Scalar& color, int thickness) {
    constexpr int SEGMENTS = 16;
    for (int i = 0; i < SEGMENTS; i += 2)
        cv::ellipse(img, center, cv::Size(radius, radius), 0, i * 360.0 / SEGMENTS, (i + 1) * 360.0 / SEGMENTS, color,
                    thickness, cv::LINE_AA);
}

}

// doc/frame_transport.md#live-frames
void draw_live_frame(cv::Mat& img, const LiveState& live, double px_per_mm) {
    const auto& t = live.target_type;
    const cv::Point center(static_cast<int>(std::lround(t.center_x_mm * px_per_mm)),
                           static_cast<int>(std::lround(t.center_y_mm * px_per_mm)));
    for (const auto& ring : t.rings) {
        const int r = static_cast<int>(std::lround(ring.radius_mm * px_per_mm));
        cv::circle(img, center, r, RING, 1, cv::LINE_AA);
    }
    const int bullet_r = std::max(3, static_cast<int>(std::lround(live.bullet_diameter_mm * px_per_mm / 2.0)));
    int number = 0;
    int newest_id = -1;
    for (const auto& h : live.holes)
        if (h.counted) newest_id = h.id;
    for (const auto& h : live.holes) {
        const cv::Point p(static_cast<int>(std::lround(h.paper_mm.x * px_per_mm)),
                          static_cast<int>(std::lround(h.paper_mm.y * px_per_mm)));
        if (!h.counted) {
            cv::circle(img, p, bullet_r, GRAY, 1, cv::LINE_AA);
            continue;
        }
        ++number;
        const bool newest = h.id == newest_id;
        cv::circle(img, p, bullet_r, newest ? NEWEST : COUNTED, newest ? 3 : 2, cv::LINE_AA);
        const std::string label = std::to_string(number);
        const double scale = std::max(0.5, bullet_r / 14.0);
        int base = 0;
        const cv::Size ts = cv::getTextSize(label, cv::FONT_HERSHEY_SIMPLEX, scale, 1, &base);
        cv::putText(img, label, p + cv::Point(bullet_r + 3, ts.height / 2), cv::FONT_HERSHEY_SIMPLEX, scale,
                    newest ? NEWEST : COUNTED, 1, cv::LINE_AA);
    }
    for (const auto& c : live.candidates) {
        const cv::Point p(static_cast<int>(std::lround(c.paper_mm.x * px_per_mm)),
                          static_cast<int>(std::lround(c.paper_mm.y * px_per_mm)));
        dashed_circle(img, p, bullet_r, CANDIDATE, 1);
    }
}

}
