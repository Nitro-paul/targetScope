// doc/programs/image_processor.md#live-mode
#pragma once

#include <functional>
#include <nlohmann/json.hpp>
#include <opencv2/core.hpp>
#include <optional>
#include <string>

#include "state.hpp"

namespace image_processor {

struct LiveEvents {
    std::function<void(const std::string& type, const nlohmann::json& payload)> publish;
};

class LivePipeline {
   public:
    LivePipeline(ProcessorState& state, LiveEvents events);

    targetscope::model::CalibrationStatus start(const targetscope::model::TargetType& t, double bullet_diameter_mm);
    targetscope::model::CalibrationStatus calibrate(const targetscope::model::Corners& corners);
    void set_counting(bool counting);
    void uncount_hole(int hole_id);
    nlohmann::json new_paper(int from_live_paper);
    void stop();
    targetscope::model::CalibrationStatus status() const;

    // doc/programs/image_processor.md#error-handling
    void frames_stopped();

    std::optional<cv::Mat> process(const cv::Mat& raw, std::uint64_t seq);
    double px_per_mm() const;

   private:
    struct Warped {
        cv::Mat bgr;
        cv::Mat gray;
    };
    void search(const cv::Mat& raw);
    void install_calibration(const std::vector<cv::Point2f>& frame_corners, Method method);
    void supervise(const cv::Mat& raw, const Warped& now);
    Warped warp(const cv::Mat& raw) const;
    cv::Mat corrected(const cv::Mat& raw) const;
    const LensCorrector* active_lens(cv::Size frame) const;
    void initial_scan(const Warped& w);
    void merge_scan(const Warped& w);
    bool is_new_paper(const Warped& w) const;
    std::string save_previous_paper();
    void replace_paper(const Warped& w);
    void emit_status();
    void set_message(const std::string& message);
    void detect_new(const Warped& w);
    cv::Size live_size() const;
    ProcessorState& state_;
    LiveEvents events_;
};

}
