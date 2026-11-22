// doc/interfaces.md#camera_reader
#include "request_handler.hpp"

#include "camera_controls.hpp"
#include "targetscope/messages.hpp"
#include "v4l2_devices.hpp"

namespace camera_reader {

using nlohmann::json;
using targetscope::ErrorCode;
using targetscope::ServiceError;

RequestHandler::RequestHandler(CaptureLoop& loop, targetscope::Bus& bus, targetscope::SettingsCache& settings)
    : loop_(loop), bus_(bus), settings_(settings) {}

// doc/interfaces.md#camera_status
json RequestHandler::status() const {
    CameraStatus s = loop_.status();
    targetscope::messages::CameraStatusResponse r;
    r.state = to_string(s.state);
    r.camera_source = s.camera_source;
    if (!s.device.empty()) r.device = s.device;
    if (!s.camera_key.empty()) r.camera_key = s.camera_key;
    if (s.info) {
        r.width_px = s.info->width_px;
        r.height_px = s.info->height_px;
        r.fps = s.info->fps;
    }
    r.frames_published = static_cast<std::int64_t>(s.frames_published);
    return r;
}

CameraStatus RequestHandler::connected_camera() const {
    CameraStatus s = loop_.status();
    if (s.state != CameraState::Connected) throw ServiceError(ErrorCode::Unavailable, "no camera is connected");
    return s;
}

// doc/programs/camera_reader.md#camera-controls
void RequestHandler::store_control(const CameraStatus& camera, std::int64_t id, std::int64_t value) {
    std::lock_guard lock(mutex_);
    json all = json::object();
    if (auto current = settings_.get("camera_controls")) {
        try {
            all = json::parse(*current);
        } catch (const json::exception&) {
        }
        if (!all.is_object()) all = json::object();
    }
    all[camera.camera_key][std::to_string(id)] = value;
    targetscope::model::Setting setting;
    setting.name = "camera_controls";
    setting.display_name = "Camera controls";
    setting.active = true;
    setting.value = all.dump();
    bus_.request(targetscope::messages::SaveSetting{setting});
}

// doc/programs/camera_reader.md#camera-controls
void RequestHandler::reset_controls() {
    CameraStatus s = loop_.status();
    if (s.state != CameraState::Connected || s.device.empty()) return;
    CameraControls(s.device).reset_to_defaults();
}

json RequestHandler::operator()(const std::string& type, const json& payload) {
    if (type == "camera_status") return status();

    if (type == "camera_list_devices") {
        json devices = json::array();
        for (auto& d : list_capture_devices()) {
            json formats = json::array();
            for (auto& f : d.formats)
                formats.push_back({{"width_px", f.width_px}, {"height_px", f.height_px}, {"fps", f.fps}});
            devices.push_back({{"path", d.path}, {"name", d.name}, {"bus_info", d.bus_info}, {"formats", formats}});
        }
        return {{"devices", devices}};
    }

    if (type == "camera_list_controls") {
        CameraStatus s = connected_camera();
        if (s.camera_source != "v4l2" || s.device.empty()) return {{"controls", json::array()}};
        return {{"controls", CameraControls(s.device).list()}};
    }

    if (type == "camera_set_control") {
        CameraStatus s = connected_camera();
        if (!payload.contains("id") || !payload["id"].is_number_integer() || !payload.contains("value") ||
            !payload["value"].is_number_integer())
            throw ServiceError(ErrorCode::ValidationFailed, "id and value must be integers");
        if (s.camera_source != "v4l2" || s.device.empty())
            throw ServiceError(ErrorCode::ValidationFailed, "this camera has no controls");
        const auto id = payload["id"].get<std::int64_t>();
        const auto value = payload["value"].get<std::int64_t>();
        auto controls = CameraControls(s.device).set(id, value);
        store_control(s, id, value);
        return {{"controls", controls}};
    }

    throw ServiceError(ErrorCode::InvalidMessage, "unknown request type " + type);
}

}
