// doc/interfaces.md#message-format
#include "targetscope/messages.hpp"

#include <array>
#include <cstdio>
#include <random>

namespace targetscope {

namespace {
constexpr std::array<const char*, 11> CODES = {"invalid_message", "validation_failed", "not_found",         "in_use",
                                               "setting_not_set", "invalid_state",     "unavailable",       "processing_failed",
                                               "io_error",        "database_error",    "timeout"};

ServiceError invalid(const std::string& msg) { return ServiceError(ErrorCode::InvalidMessage, msg); }
}  // namespace

const char* to_string(ErrorCode code) { return CODES[static_cast<std::size_t>(code)]; }

std::optional<ErrorCode> error_code_from(const std::string& s) {
    for (std::size_t i = 0; i < CODES.size(); ++i)
        if (s == CODES[i]) return static_cast<ErrorCode>(i);
    return std::nullopt;
}

// doc/interfaces.md#request-envelope
std::string new_uuid() {
    static thread_local std::mt19937_64 rng{std::random_device{}()};
    std::uniform_int_distribution<std::uint64_t> d;
    std::uint64_t a = d(rng), b = d(rng);
    a = (a & 0xFFFFFFFFFFFF0FFFull) | 0x0000000000004000ull;
    b = (b & 0x3FFFFFFFFFFFFFFFull) | 0x8000000000000000ull;
    char buf[37];
    std::snprintf(buf, sizeof buf, "%08x-%04x-%04x-%04x-%012llx", static_cast<unsigned>(a >> 32), static_cast<unsigned>((a >> 16) & 0xFFFF),
                  static_cast<unsigned>(a & 0xFFFF), static_cast<unsigned>(b >> 48), static_cast<unsigned long long>(b & 0xFFFFFFFFFFFFull));
    return buf;
}

json to_json(const RequestEnvelope& e) {
    json j{{"type", e.type}, {"version", e.version}, {"request_id", e.request_id}, {"payload", e.payload}};
    if (e.reply_to) j["reply_to"] = *e.reply_to;
    return j;
}

json to_json(const ResponseEnvelope& e) {
    return json{{"type", e.type},  {"version", e.version}, {"request_id", e.request_id}, {"status", e.ok ? "ok" : "error"},
                {"error", e.error}, {"payload", e.payload}};
}

json to_json(const EventEnvelope& e) {
    return json{{"type", e.type},     {"version", e.version},           {"event_id", e.event_id},
                {"source", e.source}, {"timestamp_ms", e.timestamp_ms}, {"payload", e.payload}};
}

// doc/interfaces.md#error-codes
RequestEnvelope parse_request(const json& j) {
    if (!j.is_object()) throw invalid("request is not a JSON object");
    RequestEnvelope e;
    try {
        e.type = j.at("type").get<std::string>();
        e.version = j.at("version").get<int>();
        e.request_id = j.at("request_id").get<std::string>();
        if (j.contains("reply_to") && !j["reply_to"].is_null()) e.reply_to = j["reply_to"].get<std::string>();
        e.payload = j.at("payload");
    } catch (const json::exception& ex) {
        throw invalid(ex.what());
    }
    if (!e.payload.is_object()) throw ServiceError(ErrorCode::ValidationFailed, "payload must be an object");
    return e;
}

ResponseEnvelope parse_response(const json& j) {
    ResponseEnvelope e;
    e.type = j.at("type").get<std::string>();
    e.version = j.at("version").get<int>();
    e.request_id = j.at("request_id").get<std::string>();
    e.ok = j.at("status").get<std::string>() == "ok";
    e.error = j.value("error", json(nullptr));
    e.payload = j.value("payload", json(nullptr));
    return e;
}

EventEnvelope parse_event(const json& j) {
    EventEnvelope e;
    e.type = j.at("type").get<std::string>();
    e.version = j.at("version").get<int>();
    e.event_id = j.at("event_id").get<std::string>();
    e.source = j.at("source").get<std::string>();
    e.timestamp_ms = j.at("timestamp_ms").get<std::int64_t>();
    e.payload = j.at("payload");
    return e;
}

ResponseEnvelope ok_response(const RequestEnvelope& req, json payload) {
    return ResponseEnvelope{req.type + "_response", req.version, req.request_id, true, nullptr, std::move(payload)};
}

ResponseEnvelope error_response(const RequestEnvelope& req, const ServiceError& e) {
    json error{{"code", to_string(e.code)}, {"message", e.what()}};
    if (!e.details.is_null()) error["details"] = e.details;
    return ResponseEnvelope{req.type + "_response", req.version, req.request_id, false, error, nullptr};
}

namespace messages {

using model::get_optional;
using model::optional_json;

void to_json(json& j, const CameraStatusResponse& v) {
    j = json{{"state", v.state},
             {"camera_source", v.camera_source},
             {"device", optional_json(v.device)},
             {"camera_key", optional_json(v.camera_key)},
             {"width_px", optional_json(v.width_px)},
             {"height_px", optional_json(v.height_px)},
             {"fps", optional_json(v.fps)},
             {"frames_published", v.frames_published}};
}

void from_json(const json& j, CameraStatusResponse& v) {
    j.at("state").get_to(v.state);
    j.at("camera_source").get_to(v.camera_source);
    get_optional(j, "device", v.device);
    get_optional(j, "camera_key", v.camera_key);
    get_optional(j, "width_px", v.width_px);
    get_optional(j, "height_px", v.height_px);
    get_optional(j, "fps", v.fps);
    v.frames_published = j.value("frames_published", std::int64_t{0});
}

void to_json(json& j, const CameraStateChanged& v) {
    j = json{{"state", v.state},
             {"message", v.message},
             {"camera_key", optional_json(v.camera_key)},
             {"width_px", optional_json(v.width_px)},
             {"height_px", optional_json(v.height_px)}};
}

void from_json(const json& j, CameraStateChanged& v) {
    j.at("state").get_to(v.state);
    v.message = j.value("message", std::string{});
    get_optional(j, "camera_key", v.camera_key);
    get_optional(j, "width_px", v.width_px);
    get_optional(j, "height_px", v.height_px);
}

void to_json(json& j, const ShotDetected& v) {
    j = json{{"hole_id", v.hole_id},     {"live_paper", v.live_paper}, {"angle_turns", v.angle_turns},
             {"radius_mm", v.radius_mm}, {"confidence", v.confidence}, {"frame_seq", v.frame_seq}};
}

void to_json(json& j, const PaperReplaced& v) {
    j = json{{"live_paper", v.live_paper}, {"previous_paper_staged_file", optional_json(v.previous_paper_staged_file)}};
}

}  // namespace messages
}  // namespace targetscope
