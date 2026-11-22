// doc/programs/image_processor.md#architecture
#include <spdlog/spdlog.h>

#include "photo_jobs.hpp"
#include "processing_thread.hpp"
#include "request_handler.hpp"
#include "targetscope/frames.hpp"
#include "targetscope/program.hpp"
#include "targetscope/settings.hpp"

using namespace image_processor;
using nlohmann::json;

int main() {
    auto started = targetscope::program::start("image_processor");
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

    const auto data_dir = started.config.data_dir;
    RuntimeProvider runtime = [&settings, data_dir] {
        Runtime r;
        r.live_long_side_px = settings->get_parsed<int>("live_long_side_px").value_or(1200);
        r.capture_long_side_px = settings->get_parsed<int>("capture_long_side_px").value_or(2400);
        r.confirm_frames = settings->get_parsed<int>("detection_confirm_frames").value_or(5);
        r.staging_dir = settings->get_path("staging_dir").value_or(data_dir / "staging");
        return r;
    };

    LiveEvents events;
    events.publish = [&bus](const std::string& type, const json& payload) { bus.publish(type, payload); };
    ProcessingThread processing(events, runtime, targetscope::frames::FRAMES_RAW, targetscope::frames::FRAMES_LIVE);
    PhotoJobs photos(runtime);
    RequestHandler handler(processing, photos, bus, *settings);
    processing.start();

    const std::string after = bus.newest_event_id("camera_reader");
    std::atomic<bool> cancel{false};
    auto learn = [&](const std::optional<std::string>& key, std::optional<int> w, std::optional<int> h) {
        CameraInfo c;
        c.camera_key = key;
        c.width_px = w.value_or(0);
        c.height_px = h.value_or(0);
        handler.set_camera(c);
    };
    std::thread events_thread([&] {
        bus.read_events(
            "camera_reader", after,
            [&](const std::string& type, const json& p) {
                if (type != "camera_state_changed") return;
                std::optional<std::string> key;
                if (p.contains("camera_key") && p["camera_key"].is_string()) key = p["camera_key"].get<std::string>();
                std::optional<int> w, h;
                if (p.contains("width_px") && p["width_px"].is_number_integer()) w = p["width_px"].get<int>();
                if (p.contains("height_px") && p["height_px"].is_number_integer()) h = p["height_px"].get<int>();
                learn(key, w, h);
            },
            &cancel);
    });
    std::thread status_thread([&] {
        for (int i = 0; i < 30 && !cancel && !targetscope::program::shutting_down(); ++i) {
            try {
                auto s = bus.request(targetscope::messages::CameraStatus{});
                learn(s.camera_key, s.width_px ? std::optional<int>(static_cast<int>(*s.width_px)) : std::nullopt,
                      s.height_px ? std::optional<int>(static_cast<int>(*s.height_px)) : std::nullopt);
                return;
            } catch (const std::exception&) {
                targetscope::program::wait_for_shutdown(std::chrono::seconds(1));
            }
        }
    });
    settings->watch({"lens_calibrations"}, [&] { handler.refresh_lens(); });

    bus.serve([&](const std::string& type, const json& payload) { return handler(type, payload); });

    while (!targetscope::program::wait_for_shutdown(std::chrono::milliseconds(200))) {
    }
    spdlog::info("shutting down");
    cancel = true;
    bus.stop();
    processing.stop();
    if (events_thread.joinable()) events_thread.join();
    if (status_thread.joinable()) status_thread.join();
    return 0;
}
