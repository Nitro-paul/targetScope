// doc/programs/image_processor.md#architecture
#pragma once

#include <chrono>
#include <cstdint>
#include <filesystem>
#include <functional>
#include <memory>
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
    std::vector<cv::Point2f> frame_corners;
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
    double area_mm2 = 0;
    double major_mm = 0;
    double orientation_rad = 0;
    bool merged = false;
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
    cv::Mat warp_map;
    bool lens_applied = false;
    int search_failures = 0;
    long frame_counter = 0;
    std::uint64_t frame_seq = 0;
    std::chrono::steady_clock::time_point last_search{};
    bool frames_gone = false;
    std::string message;
    std::int64_t updated_ms = 0;
    int next_candidate_id = 1;
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

// doc/programs/image_processor.md#architecture
struct Runtime {
    int live_long_side_px = 1200;
    int capture_long_side_px = 2400;
    int confirm_frames = 5;
    std::chrono::milliseconds search_interval{1000};
    std::filesystem::path staging_dir;
};
using RuntimeProvider = std::function<Runtime()>;

class LensCorrector;

struct ProcessorState {
    Runtime runtime;
    std::chrono::steady_clock::time_point last_frame_time{};
    std::shared_ptr<LensCorrector> corrector;
    cv::Size lens_size;
    cv::Mat newest_raw;
    std::uint64_t newest_raw_seq = 0;
    std::optional<LiveState> live;
    CameraInfo camera;
    std::optional<LensCalibration> lens;
    std::optional<LensCalibrationSession> collecting;
};

}
