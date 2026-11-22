// doc/programs/camera_reader.md#architecture
#include <spdlog/spdlog.h>

#include <map>

#include "camera_controls.hpp"
#include "camera_identity.hpp"
#include "capture_loop.hpp"
#include "request_handler.hpp"
#include "rtsp_source.hpp"
#include "targetscope/frames.hpp"
#include "targetscope/program.hpp"
#include "targetscope/settings.hpp"
#include "v4l2_source.hpp"

using namespace camera_reader;
using nlohmann::json;

namespace {

// doc/programs/camera_reader.md#capture-loop
SourceChoice choose_source(targetscope::SettingsCache& settings) {
    SourceChoice c;
    c.camera_source = settings.get("camera_source").value_or("v4l2");
    if (c.camera_source == "none") {
        c.message = "camera_source is none";
        return c;
    }
    if (c.camera_source == "rtsp") {
        auto url = settings.get("camera_rtsp_url");
        if (!url || url->empty()) {
            c.message = "the setting camera_rtsp_url is not set";
            return c;
        }
        c.camera_key = camera_key_rtsp(*url);
        c.source = std::make_unique<RtspSource>(*url);
        return c;
    }
    V4l2Settings v;
    v.bus_info = settings.get("camera_device").value_or("");
    v.width_px = settings.get_parsed<int>("camera_width_px").value_or(1920);
    v.height_px = settings.get_parsed<int>("camera_height_px").value_or(1080);
    v.fps = settings.get_parsed<double>("camera_fps").value_or(30);
    c.camera_key = camera_key_v4l2(v.bus_info);
    c.source = std::make_unique<V4l2Source>(v);
    return c;
}

std::map<std::int64_t, std::int64_t> stored_controls(targetscope::SettingsCache& settings, const std::string& key) {
    std::map<std::int64_t, std::int64_t> out;
    auto text = settings.get("camera_controls");
    if (!text) return out;
    try {
        auto all = json::parse(*text);
        if (all.is_object() && all.contains(key) && all[key].is_object())
            for (auto& [id, v] : all[key].items()) out[std::stoll(id)] = v.get<std::int64_t>();
    } catch (const std::exception& e) {
        spdlog::warn("setting camera_controls is not valid error={}", e.what());
    }
    return out;
}

}

int main() {
    auto started = targetscope::program::start("camera_reader");
    if (!started.bus) return 0;
    targetscope::Bus& bus = *started.bus;

    std::unique_ptr<targetscope::SettingsCache> settings;
    auto backoff = std::chrono::milliseconds(100);
    while (!settings && !targetscope::program::shutting_down()) {
        try {
            settings = std::make_unique<targetscope::SettingsCache>(bus);
        } catch (const std::exception& e) {
            spdlog::warn("loading settings failed error={}", e.what());
            targetscope::program::wait_for_shutdown(backoff);
            backoff = std::min(backoff * 2, std::chrono::milliseconds(5000));
        }
    }
    if (!settings) return 0;

    CaptureLoop loop([&] { return choose_source(*settings); },
                     [&](const CameraStatus& s) {
                         targetscope::messages::CameraStateChanged e;
                         e.state = to_string(s.state);
                         e.message = s.message;
                         if (!s.camera_key.empty()) e.camera_key = s.camera_key;
                         if (s.info) {
                             e.width_px = s.info->width_px;
                             e.height_px = s.info->height_px;
                         }
                         bus.publish("camera_state_changed", e);
                     },
                     targetscope::frames::FRAMES_RAW);
    loop.set_opened_callback([&](FrameSource& source, const CameraStatus& s) {
        if (!source.has_controls() || s.device.empty()) return;
        CameraControls(s.device).apply(stored_controls(*settings, s.camera_key));
    });
    RequestHandler handler(loop, bus, *settings);

    // doc/programs/camera_reader.md#responsibilities
    settings->watch(
        {"camera_source", "camera_device", "camera_rtsp_url", "camera_width_px", "camera_height_px", "camera_fps"},
        [&] { loop.reopen(); });
    settings->watch({"camera_controls"}, [&] {
        auto text = settings->get("camera_controls");
        try {
            if (text && json::parse(*text) == json::object()) handler.reset_controls();
        } catch (const std::exception& e) {
            spdlog::warn("resetting the camera controls failed error={}", e.what());
        }
    });

    loop.start();
    bus.serve([&](const std::string& type, const json& payload) { return handler(type, payload); });

    while (!targetscope::program::wait_for_shutdown(std::chrono::milliseconds(200))) {
    }
    spdlog::info("shutting down");
    bus.stop();
    loop.stop();
    return 0;
}
