// Live mode per frame (doc/programs/image_processor.md#live-mode): calibration search and
// supervision, initial hole scan, new holes, paper change (requested and automatic), live frame.
#pragma once

#include <functional>
#include <optional>
#include <string>

#include <nlohmann/json.hpp>
#include <opencv2/core.hpp>

#include "state.hpp"

namespace image_processor {

// Events produced while processing (sent by the Redis thread).
struct LiveEvents {
    std::function<void(const std::string& type, const nlohmann::json& payload)> publish;
};

class LivePipeline {
public:
    LivePipeline(ProcessorState& state, LiveEvents events);

    void start(const targetscope::model::TargetType& t, double bullet_diameter_mm);  // live_start
    void calibrate(const targetscope::model::Corners& corners);                      // live_calibrate
    void set_counting(bool counting);                                                // live_set_counting
    void uncount_hole(int hole_id);                                                  // live_uncount_hole
    // live_new_paper: returns {changed, live_paper, previous_paper_staged_file}
    nlohmann::json new_paper(int from_live_paper);
    void stop();                                                                     // live_stop
    targetscope::model::CalibrationStatus status() const;

    // One raw frame (already lens-corrected if a calibration applies); returns the live frame to publish.
    std::optional<cv::Mat> process(const cv::Mat& raw);

private:
    void initial_hole_scan(const cv::Mat& transformed_gray);
    void supervise(const cv::Mat& raw);
    bool is_new_paper(const cv::Mat& transformed_gray) const;  // at most 20 % of known holes visible
    ProcessorState& state_;
    LiveEvents events_;
};

}  // namespace image_processor
