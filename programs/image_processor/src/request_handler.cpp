// doc/interfaces.md#image_processor
#include "request_handler.hpp"

#include "capture.hpp"
#include "lens_calibration.hpp"
#include "targetscope/messages.hpp"
#include "targetscope/targets.hpp"

namespace image_processor {

using nlohmann::json;
using targetscope::ErrorCode;
using targetscope::ServiceError;

namespace {

constexpr const char* LENS_SETTING = "lens_calibrations";

// doc/programs/image_processor.md#lens-calibration
CameraInfo current_camera(const ProcessorState& s) {
    CameraInfo c = s.camera;
    if (!s.newest_raw.empty()) {
        c.width_px = s.newest_raw.cols;
        c.height_px = s.newest_raw.rows;
    }
    return c;
}

void discard_if_camera_changed(ProcessorState& s) {
    if (!s.collecting) return;
    const CameraInfo now = current_camera(s);
    const auto& was = s.collecting->camera;
    if (was.camera_key != now.camera_key || was.width_px != now.width_px || was.height_px != now.height_px) {
        s.collecting->views.clear();
        s.collecting->camera = now;
    }
}

targetscope::model::TargetType parse_target_type(const json& j) {
    targetscope::model::TargetType t;
    try {
        t = j.get<targetscope::model::TargetType>();
    } catch (const json::exception& e) {
        throw ServiceError(ErrorCode::ValidationFailed, std::string("target_type: ") + e.what());
    }
    if (auto problem = targetscope::targets::validate_target_type(t))
        throw ServiceError(ErrorCode::ValidationFailed, *problem);
    t.rings = targetscope::targets::generate_rings(t);
    return t;
}

}

RequestHandler::RequestHandler(ProcessingThread& processing, PhotoJobs& photos, targetscope::Bus& bus,
                               targetscope::SettingsCache& settings)
    : processing_(processing), photos_(photos), bus_(bus), settings_(settings) {}

json RequestHandler::run(ProcessingThread::Command cmd) { return processing_.submit(std::move(cmd)).get(); }

json RequestHandler::read_lens_setting() {
    if (auto v = settings_.get(LENS_SETTING)) {
        try {
            auto j = json::parse(*v);
            if (j.is_object()) return j;
        } catch (const json::exception&) {
        }
    }
    return json::object();
}

// doc/database.md#known-settings
void RequestHandler::store_lens_setting(const json& all) {
    targetscope::model::Setting s;
    s.name = LENS_SETTING;
    s.display_name = "Lens calibrations";
    s.active = true;
    s.value = all.dump();
    bus_.request(targetscope::messages::SaveSetting{s});
}

// doc/programs/image_processor.md#lens-calibration
void RequestHandler::set_camera(const CameraInfo& camera) {
    run([camera](ProcessorState& s, LivePipeline&) {
        s.camera = camera;
        discard_if_camera_changed(s);
        return json::object();
    });
    refresh_lens();
}

void RequestHandler::refresh_lens() {
    const json all = read_lens_setting();
    run([all](ProcessorState& s, LivePipeline&) {
        auto c = from_setting(all, s.camera);
        if (c) {
            s.lens = *c;
            s.lens_size = cv::Size(c->width_px, c->height_px);
            s.corrector = std::make_shared<LensCorrector>(*c, s.lens_size);
        } else {
            s.lens.reset();
            s.corrector.reset();
            s.lens_size = {};
        }
        return json::object();
    });
}

json RequestHandler::lens_status(ProcessorState& s) {
    discard_if_camera_changed(s);
    const CameraInfo cam = current_camera(s);
    json j{
        {"camera_key", s.camera.camera_key ? json(*s.camera.camera_key) : json(nullptr)},
        {"calibrated", s.lens.has_value()},
        {"applied", s.corrector != nullptr && cam.width_px == s.lens_size.width && cam.height_px == s.lens_size.height},
        {"rms_px", s.lens ? json(s.lens->rms_px) : json(nullptr)},
        {"width_px", s.lens ? json(s.lens->width_px) : json(nullptr)},
        {"height_px", s.lens ? json(s.lens->height_px) : json(nullptr)},
        {"collecting_views", s.collecting ? json(static_cast<int>(s.collecting->views.size())) : json(nullptr)}};
    return j;
}

json RequestHandler::operator()(const std::string& type, const json& payload) {
    // doc/interfaces.md#live-mode
    if (type == "live_start") {
        if (!payload.contains("target_type") || !payload.contains("bullet_diameter_mm") ||
            !payload["bullet_diameter_mm"].is_number())
            throw ServiceError(ErrorCode::ValidationFailed, "target_type and bullet_diameter_mm are required");
        auto t = parse_target_type(payload["target_type"]);
        const double bullet = payload["bullet_diameter_mm"].get<double>();
        if (!(bullet > 0)) throw ServiceError(ErrorCode::ValidationFailed, "bullet_diameter_mm must be above 0");
        return run([&](ProcessorState&, LivePipeline& p) { return json(p.start(t, bullet)); });
    }
    if (type == "live_calibrate") {
        if (!payload.contains("corners")) throw ServiceError(ErrorCode::ValidationFailed, "corners are required");
        targetscope::model::Corners corners;
        try {
            corners = payload["corners"].get<targetscope::model::Corners>();
        } catch (const json::exception& e) {
            throw ServiceError(ErrorCode::ValidationFailed, std::string("corners: ") + e.what());
        }
        return run([&](ProcessorState&, LivePipeline& p) { return json(p.calibrate(corners)); });
    }
    if (type == "live_set_counting") {
        if (!payload.contains("counting") || !payload["counting"].is_boolean())
            throw ServiceError(ErrorCode::ValidationFailed, "counting must be a boolean");
        const bool counting = payload["counting"];
        return run([&](ProcessorState&, LivePipeline& p) {
            p.set_counting(counting);
            return json::object();
        });
    }
    if (type == "live_uncount_hole") {
        if (!payload.contains("hole_id") || !payload["hole_id"].is_number_integer())
            throw ServiceError(ErrorCode::ValidationFailed, "hole_id must be an integer");
        const int id = payload["hole_id"];
        return run([&](ProcessorState&, LivePipeline& p) {
            p.uncount_hole(id);
            return json::object();
        });
    }
    if (type == "live_new_paper") {
        if (!payload.contains("from_live_paper") || !payload["from_live_paper"].is_number_integer())
            throw ServiceError(ErrorCode::ValidationFailed, "from_live_paper must be an integer");
        const int from = payload["from_live_paper"];
        return run([&](ProcessorState&, LivePipeline& p) { return p.new_paper(from); });
    }
    if (type == "live_stop") {
        return run([](ProcessorState&, LivePipeline& p) {
            p.stop();
            return json::object();
        });
    }
    if (type == "live_calibration_status")
        return run([](ProcessorState&, LivePipeline& p) { return json(p.status()); });

    // doc/interfaces.md#lens-calibration
    if (type.rfind("lens_calibration_", 0) == 0) {
        const std::string what = type.substr(std::string("lens_calibration_").size());
        if (what == "status") return run([this](ProcessorState& s, LivePipeline&) { return lens_status(s); });
        if (what == "start") {
            return run([](ProcessorState& s, LivePipeline&) -> json {
                if (s.live)
                    throw ServiceError(ErrorCode::InvalidState,
                                       "lens calibration isn't possible while live mode is active");
                LensCalibrationSession session;
                session.camera = current_camera(s);
                s.collecting = std::move(session);
                return {{"views", 0}};
            });
        }
        if (what == "capture") {
            return run([](ProcessorState& s, LivePipeline&) -> json {
                if (!s.collecting) throw ServiceError(ErrorCode::InvalidState, "lens calibration was not started");
                discard_if_camera_changed(s);
                if (s.newest_raw.empty() ||
                    std::chrono::steady_clock::now() - s.last_frame_time > std::chrono::seconds(2))
                    throw ServiceError(ErrorCode::Unavailable, "no camera frames");
                const bool found = capture_view(*s.collecting, s.newest_raw);
                return {{"found", found}, {"views", static_cast<int>(s.collecting->views.size())}};
            });
        }
        if (what == "finish") {
            json result = run([](ProcessorState& s, LivePipeline&) -> json {
                if (!s.collecting) throw ServiceError(ErrorCode::InvalidState, "lens calibration was not started");
                discard_if_camera_changed(s);
                if (!s.collecting->camera.camera_key)
                    throw ServiceError(ErrorCode::Unavailable, "the camera is not known");
                LensCalibration c = finish(*s.collecting);
                json entry = to_setting(c);
                json out{{"rms_px", c.rms_px},
                         {"views", static_cast<int>(s.collecting->views.size())},
                         {"key", c.camera_key},
                         {"entry", entry}};
                s.collecting.reset();
                return out;
            });
            json all = read_lens_setting();
            all[result["key"].get<std::string>()] = result["entry"];
            store_lens_setting(all);
            refresh_lens();
            return {{"rms_px", result["rms_px"]}, {"views", result["views"]}};
        }
        if (what == "cancel") {
            return run([](ProcessorState& s, LivePipeline&) {
                s.collecting.reset();
                return json::object();
            });
        }
        if (what == "delete") {
            auto key = run([](ProcessorState& s, LivePipeline&) -> json {
                return s.camera.camera_key ? json(*s.camera.camera_key) : json(nullptr);
            });
            if (key.is_string()) {
                json all = read_lens_setting();
                if (all.contains(key.get<std::string>())) {
                    all.erase(key.get<std::string>());
                    store_lens_setting(all);
                }
            }
            refresh_lens();
            return json::object();
        }
        throw ServiceError(ErrorCode::InvalidMessage, "unknown request type " + type);
    }

    // doc/interfaces.md#capture-mode
    if (type == "capture_target_image") {
        return run([](ProcessorState& s, LivePipeline&) -> json {
            return {{"staged_file", capture_target_image(s, s.runtime.staging_dir, s.runtime.capture_long_side_px)}};
        });
    }
    if (type == "capture_raw_image") {
        return run([](ProcessorState& s, LivePipeline&) -> json {
            auto [name, corrected] = capture_raw_image(s, s.runtime.staging_dir);
            return {{"staged_file", name}, {"lens_corrected", corrected}};
        });
    }
    if (type == "find_paper") return photos_.find_paper(payload).get();
    if (type == "prepare_target_photo") return photos_.prepare_target_photo(payload).get();
    if (type == "detect_shots_in_photo") return photos_.detect_shots_in_photo(payload).get();

    throw ServiceError(ErrorCode::InvalidMessage, "unknown request type " + type);
}

}
