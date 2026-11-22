// doc/programs/image_processor.md#capture-mode
#include "capture.hpp"

#include <opencv2/imgproc.hpp>

#include "lens_calibration.hpp"
#include "staging.hpp"
#include "targetscope/messages.hpp"

namespace image_processor {

using targetscope::ErrorCode;
using targetscope::ServiceError;

namespace {
constexpr int JPEG_QUALITY = 92;
constexpr auto FRAME_STALE = std::chrono::seconds(2);
}

// doc/programs/image_processor.md#capture_target_image
cv::Mat warp_at(const ProcessorState& state, const LiveState& live, int long_side_px, double* px_per_mm_out) {
    const auto& t = live.target_type;
    const double ppm = long_side_px / std::max(t.paper_width_mm, t.paper_height_mm);
    if (px_per_mm_out) *px_per_mm_out = ppm;
    const cv::Size size(static_cast<int>(std::lround(t.paper_width_mm * ppm)),
                        static_cast<int>(std::lround(t.paper_height_mm * ppm)));
    const double live_ppm = live.calibration ? live.calibration->px_per_mm : ppm;
    cv::Mat S = cv::Mat::eye(3, 3, CV_64F);
    S.at<double>(0, 0) = S.at<double>(1, 1) = ppm / live_ppm;
    cv::Mat H = S * live.last_good_homography;
    cv::Mat out;
    if (live.lens_applied && state.corrector) {
        cv::Mat map = state.corrector->combined_map(H, size);
        cv::remap(live.last_good_raw, out, map, cv::noArray(), cv::INTER_LINEAR);
    } else {
        cv::warpPerspective(live.last_good_raw, out, H, size, cv::INTER_LINEAR);
    }
    return out;
}

std::string capture_target_image(const ProcessorState& state, const std::filesystem::path& staging_dir,
                                 int capture_long_side_px) {
    if (!state.live || state.live->last_good_raw.empty() || state.live->last_good_homography.empty())
        throw ServiceError(ErrorCode::Unavailable, "there is no good frame of the current paper");
    cv::Mat img = warp_at(state, *state.live, capture_long_side_px, nullptr);
    return write_staged_jpeg(staging_dir, "target", img, JPEG_QUALITY);
}

// doc/programs/image_processor.md#capture_raw_image
std::pair<std::string, bool> capture_raw_image(const ProcessorState& state, const std::filesystem::path& staging_dir) {
    if (state.newest_raw.empty() || std::chrono::steady_clock::now() - state.last_frame_time > FRAME_STALE)
        throw ServiceError(ErrorCode::Unavailable, "no camera frames");
    const bool corrected = state.corrector && state.lens_size == state.newest_raw.size();
    cv::Mat img;
    if (corrected)
        state.corrector->correct(state.newest_raw, img);
    else
        img = state.newest_raw;
    return {write_staged_jpeg(staging_dir, "raw", img, JPEG_QUALITY), corrected};
}

}
