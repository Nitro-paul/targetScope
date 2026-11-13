// doc/programs/image_processor.md#architecture
#pragma once

#include <opencv2/core.hpp>
#include <optional>
#include <string>
#include <vector>

#include "targetscope/model.hpp"

namespace image_processor {

enum class Method { Auto, Manual };
enum class CalState { Calibrated, Lost };

struct Calibration {
    Method method = Method::Auto;
    targetscope::model::Corners corners;
    cv::Mat homography;
    double px_per_mm = 0;
    CalState state = CalState::Calibrated;
};

struct Hole {
    int id = 0;
    cv::Point2d paper_mm;
    bool counted = false;
    float confidence = 0;
};

struct Candidate {
    cv::Point2d paper_mm;
    double diameter_mm = 0;
    double roundness = 0;
    int seen_frames = 0;
    int missed_frames = 0;
};

struct LiveState {
    targetscope::model::TargetType target_type;
    double bullet_diameter_mm = 0;
    std::optional<Calibration> calibration;
    cv::Mat reference;
    std::vector<Hole> holes;
    std::vector<Candidate> candidates;
    bool counting = false;
    int next_hole_id = 1;
    int live_paper = 1;
    cv::Mat last_good_raw;
    cv::Mat last_good_homography;
    long frame_counter = 0;
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
    cv::Mat dist_coeffs;
    double rms_px = 0;
};

struct LensCalibrationSession {
    std::vector<std::vector<cv::Point2f>> views;
    CameraInfo camera;
};

struct ProcessorState {
    cv::Mat newest_raw;
    std::optional<LiveState> live;
    CameraInfo camera;
    std::optional<LensCalibration> lens;
    std::optional<LensCalibrationSession> collecting;
};

}
