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

    void start(const targetscope::model::TargetType& t, double bullet_diameter_mm);
    void calibrate(const targetscope::model::Corners& corners);
    void set_counting(bool counting);
    void uncount_hole(int hole_id);
    nlohmann::json new_paper(int from_live_paper);
    void stop();
    targetscope::model::CalibrationStatus status() const;

    std::optional<cv::Mat> process(const cv::Mat& raw);

   private:
    void initial_hole_scan(const cv::Mat& transformed_gray);
    void supervise(const cv::Mat& raw);
    bool is_new_paper(const cv::Mat& transformed_gray) const;
    ProcessorState& state_;
    LiveEvents events_;
};

}
