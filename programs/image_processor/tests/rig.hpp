// doc/testing/image_processor.md#setup
#pragma once

#include <mutex>
#include <vector>

#include "lens_calibration.hpp"
#include "live_pipeline.hpp"
#include "synthetic.hpp"
#include "targetscope/messages.hpp"
#include "targetscope/testkit/temp_data_dir.hpp"

namespace image_processor::testing {

using nlohmann::json;

struct Rig {
    targetscope::testkit::TempDataDir dirs;
    ProcessorState state;
    std::vector<std::pair<std::string, json>> events;
    LivePipeline pipeline;
    std::uint64_t seq = 0;
    std::optional<cv::Mat> last_out;

    Rig()
        : pipeline(state, LiveEvents{[this](const std::string& type, const json& payload) {
                       events.emplace_back(type, payload);
                   }}) {
        state.runtime.staging_dir = dirs.staging_dir;
        state.runtime.search_interval = std::chrono::milliseconds(0);
    }

    // doc/testing/image_processor.md#live-mode-start-and-calibration-integration
    void feed(const cv::Mat& frame, int count = 1) {
        for (int i = 0; i < count; ++i) {
            state.newest_raw = frame;
            state.newest_raw_seq = seq;
            state.last_frame_time = std::chrono::steady_clock::now();
            last_out = pipeline.process(frame, seq++);
        }
    }

    void start(const Scene& s, bool counting = false) {
        pipeline.start(s.target, s.bullet_mm);
        if (counting) {
            feed(render(s), 2);
            pipeline.set_counting(true);
        }
    }

    std::vector<json> events_of(const std::string& type) const {
        std::vector<json> out;
        for (auto& [t, p] : events)
            if (t == type) out.push_back(p);
        return out;
    }

    void install_lens(double k1, cv::Size image) {
        LensCalibration c;
        c.camera_key = "cam";
        c.width_px = image.width;
        c.height_px = image.height;
        c.camera_matrix = camera_matrix(image);
        c.dist_coeffs = cv::Mat(cv::Matx<double, 5, 1>(k1, 0, 0, 0, 0));
        state.lens = c;
        state.lens_size = image;
        state.corrector = std::make_shared<LensCorrector>(c, image);
    }
};

inline double dist_to_truth(const cv::Point2d& p, const cv::Point2d& truth) {
    return std::hypot(p.x - truth.x, p.y - truth.y);
}

}
