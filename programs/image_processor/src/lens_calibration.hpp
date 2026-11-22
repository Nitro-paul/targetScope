// doc/programs/image_processor.md#lens-calibration
#pragma once

#include <nlohmann/json.hpp>
#include <opencv2/core.hpp>
#include <optional>
#include <vector>

#include "state.hpp"

namespace image_processor {

inline constexpr int BOARD_COLS = 9;
inline constexpr int BOARD_ROWS = 6;
inline constexpr int MIN_VIEWS = 10;

bool capture_view(LensCalibrationSession& session, const cv::Mat& raw);

LensCalibration finish(const LensCalibrationSession& session);

nlohmann::json to_setting(const LensCalibration& c);
std::optional<LensCalibration> from_setting(const nlohmann::json& all, const CameraInfo& camera);

class LensCorrector {
   public:
    explicit LensCorrector(const LensCalibration& c, cv::Size size);
    void correct(const cv::Mat& raw, cv::Mat& out) const;

    // doc/programs/image_processor.md#start-and-calibration
    std::vector<cv::Point2f> to_corrected(const std::vector<cv::Point2f>& raw_points) const;
    std::vector<cv::Point2f> to_raw(const std::vector<cv::Point2f>& corrected_points) const;

    // doc/programs/image_processor.md#lens-calibration
    cv::Mat combined_map(const cv::Mat& homography_corrected_to_paper, cv::Size out) const;

   private:
    cv::Mat camera_matrix_, dist_coeffs_;
    cv::Mat map1_, map2_;
};

}
