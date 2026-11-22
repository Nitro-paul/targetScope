// doc/programs/camera_reader.md#architecture
#pragma once

#include <nlohmann/json.hpp>
#include <string>

#include "capture_loop.hpp"
#include "targetscope/bus.hpp"
#include "targetscope/settings.hpp"

namespace camera_reader {

class RequestHandler {
   public:
    RequestHandler(CaptureLoop& loop, targetscope::Bus& bus, targetscope::SettingsCache& settings);
    nlohmann::json operator()(const std::string& type, const nlohmann::json& payload);

    nlohmann::json status() const;
    void reset_controls();

   private:
    CameraStatus connected_camera() const;
    void store_control(const CameraStatus& camera, std::int64_t id, std::int64_t value);

    CaptureLoop& loop_;
    targetscope::Bus& bus_;
    targetscope::SettingsCache& settings_;
    mutable std::mutex mutex_;
};

}
