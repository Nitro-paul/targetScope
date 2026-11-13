// doc/interfaces.md#image_processor
#pragma once

#include <nlohmann/json.hpp>
#include <string>

#include "photo_jobs.hpp"
#include "processing_thread.hpp"

namespace image_processor {

class RequestHandler {
   public:
    RequestHandler(ProcessingThread& processing, PhotoJobs& photos);
    nlohmann::json operator()(const std::string& type, const nlohmann::json& payload);

   private:
    ProcessingThread& processing_;
    PhotoJobs& photos_;
};

}
