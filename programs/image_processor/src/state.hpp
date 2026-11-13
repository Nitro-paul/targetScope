// State of the processing thread (doc/programs/image_processor.md#architecture).
#pragma once

#include <optional>
#include <string>
#include <vector>

#include <opencv2/core.hpp>

#include "targetscope/model.hpp"

namespace image_processor {

enum class Method { Auto, Manual };
enum class CalState { Calibrated, Lost };

struct Calibration {
    Method method = Method::Auto;
    targetscope::model::Corners corners;  // paper corners in raw camera pixels (as clicked / reported)
    cv::Mat homography;                   // raw -> transformed live image
    double px_per_mm = 0;
    CalState state = CalState::Calibrated;
};

struct Hole {
    int id = 0;            // unique within one live mode, sent as hole_id
    cv::Point2d paper_mm;
    bool counted = false;  // true: session shot (highlighted), false: old/uncounted (gray)
    float confidence = 0;
};  // shot number in the live view = position among counted holes of the current paper

struct Candidate {
    cv::Point2d paper_mm;
    double diameter_mm = 0;
    double roundness = 0;
    int seen_frames = 0;    // consecutive frames; confirmed at detection_confirm_frames
    int missed_frames = 0;  // dropped after 3
};

struct LiveState {
    targetscope::model::TargetType target_type;
    double bullet_diameter_mm = 0;
    std::optional<Calibration> calibration;
    cv::Mat reference;                  // transformed grayscale image incl. all known holes
    std::vector<Hole> holes;            // all confirmed holes
    std::vector<Candidate> candidates;  // not yet confirmed
    bool counting = false;
    int next_hole_id = 1;
    int live_paper = 1;                 // +1 at every paper change; sent with shot_detected
    cv::Mat last_good_raw;              // newest frame with the current paper fully visible, and
    cv::Mat last_good_homography;       //   its homography
    long frame_counter = 0;             // supervision every 30th frame
};

struct CameraInfo {
    std::optional<std::string> camera_key;
    int width_px = 0;
    int height_px = 0;
};

struct LensCalibration {
    std::string camera_key;
    int width_px = 0;
    int height_px = 0;
    cv::Mat camera_matrix;
    cv::Mat dist_coeffs;  // k1, k2, p1, p2, k3
    double rms_px = 0;
};

struct LensCalibrationSession {
    std::vector<std::vector<cv::Point2f>> views;  // checkerboard corners per view (9 × 6)
    CameraInfo camera;                            // views are discarded if the camera changes
};

struct ProcessorState {
    cv::Mat newest_raw;                               // newest camera frame as received
    std::optional<LiveState> live;                    // set between live_start and live_stop
    CameraInfo camera;
    std::optional<LensCalibration> lens;              // applied calibration of the current camera
    std::optional<LensCalibrationSession> collecting;  // between lens_calibration_start and _finish/_cancel
};

}  // namespace image_processor
