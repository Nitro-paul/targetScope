// doc/interfaces.md#image_processor
#pragma once

#include <nlohmann/json.hpp>
#include <string>

#include "photo_jobs.hpp"
#include "processing_thread.hpp"
#include "targetscope/bus.hpp"
#include "targetscope/settings.hpp"

namespace image_processor {

class RequestHandler {
   public:
    RequestHandler(ProcessingThread& processing, PhotoJobs& photos, targetscope::Bus& bus,
                   targetscope::SettingsCache& settings);
    nlohmann::json operator()(const std::string& type, const nlohmann::json& payload);

    // doc/programs/image_processor.md#lens-calibration
    void set_camera(const CameraInfo& camera);
    void refresh_lens();

   private:
    nlohmann::json run(ProcessingThread::Command cmd);
    nlohmann::json lens_status(ProcessorState& state);
    void store_lens_setting(const nlohmann::json& all);
    nlohmann::json read_lens_setting();

    ProcessingThread& processing_;
    PhotoJobs& photos_;
    targetscope::Bus& bus_;
    targetscope::SettingsCache& settings_;
};

}
