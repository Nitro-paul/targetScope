// image_processor:requests: live_*, lens_calibration_*, capture_*, find_paper,
// prepare_target_photo, detect_shots_in_photo (doc/interfaces.md#image_processor).
#pragma once

#include <string>

#include <nlohmann/json.hpp>

#include "photo_jobs.hpp"
#include "processing_thread.hpp"

namespace image_processor {

class RequestHandler {
public:
    RequestHandler(ProcessingThread& processing, PhotoJobs& photos);
    // Called by Bus::serve; throws targetscope::ServiceError.
    nlohmann::json operator()(const std::string& type, const nlohmann::json& payload);
private:
    ProcessingThread& processing_;
    PhotoJobs& photos_;
};

}  // namespace image_processor
