// doc/programs/camera_reader.md#architecture
#pragma once

#include <nlohmann/json.hpp>
#include <string>

#include "capture_loop.hpp"

namespace camera_reader {

class RequestHandler {
   public:
    explicit RequestHandler(CaptureLoop& loop);
    nlohmann::json operator()(const std::string& type, const nlohmann::json& payload);

   private:
    CaptureLoop& loop_;
};

}
