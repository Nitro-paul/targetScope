// camera_reader:requests: camera_status, camera_list_devices, camera_list_controls, camera_set_control.
#pragma once

#include <string>

#include <nlohmann/json.hpp>

#include "capture_loop.hpp"

namespace camera_reader {

class RequestHandler {
public:
    explicit RequestHandler(CaptureLoop& loop);
    // Called by Bus::serve; throws targetscope::ServiceError.
    nlohmann::json operator()(const std::string& type, const nlohmann::json& payload);
private:
    CaptureLoop& loop_;
};

}  // namespace camera_reader
