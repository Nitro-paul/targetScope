// doc/programs/image_processor.md#lens-calibration
#pragma once

#include <nlohmann/json.hpp>
#include <opencv2/core.hpp>
#include <optional>

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
    explicit LensCorrector(const LensCalibration& c);
    void correct(const cv::Mat& raw, cv::Mat& out) const;

   private:
    cv::Mat map1_, map2_;
};

}
