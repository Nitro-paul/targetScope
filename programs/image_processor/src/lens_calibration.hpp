// Lens calibration per camera (doc/programs/image_processor.md#lens-calibration):
// collect checkerboard views (9 × 6 inner corners, findChessboardCornersSB), calibrateCamera,
// store in the setting lens_calibrations under camera_key; apply with a remap table.
#pragma once

#include <optional>

#include <nlohmann/json.hpp>
#include <opencv2/core.hpp>

#include "state.hpp"

namespace image_processor {

inline constexpr int BOARD_COLS = 9;
inline constexpr int BOARD_ROWS = 6;
inline constexpr int MIN_VIEWS = 10;

// lens_calibration_capture on the newest raw frame (not corrected); true if the board was found.
bool capture_view(LensCalibrationSession& session, const cv::Mat& raw);

// lens_calibration_finish; throws validation_failed with fewer than MIN_VIEWS views.
LensCalibration finish(const LensCalibrationSession& session);

// Setting value lens_calibrations <-> calibrations per camera_key
nlohmann::json to_setting(const LensCalibration& c);
std::optional<LensCalibration> from_setting(const nlohmann::json& all, const CameraInfo& camera);

// Correction map (initUndistortRectifyMap), built once per camera and resolution.
class LensCorrector {
public:
    explicit LensCorrector(const LensCalibration& c);
    void correct(const cv::Mat& raw, cv::Mat& out) const;
private:
    cv::Mat map1_, map2_;
};

}  // namespace image_processor
