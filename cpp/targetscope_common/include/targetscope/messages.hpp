// doc/interfaces.md#message-format
// doc/shared_libraries.md#c-library-targetscope_common_cpp
#pragma once

#include <chrono>
#include <cstdint>
#include <optional>
#include <stdexcept>
#include <string>
#include <vector>

#include <nlohmann/json.hpp>

#include "targetscope/model.hpp"

namespace targetscope {

using nlohmann::json;
using namespace std::chrono_literals;

// doc/interfaces.md#request-envelope
inline constexpr int MESSAGE_VERSION = 1;

// doc/interfaces.md#error-codes
enum class ErrorCode {
    InvalidMessage,
    ValidationFailed,
    NotFound,
    InUse,
    SettingNotSet,
    InvalidState,
    Unavailable,
    ProcessingFailed,
    IoError,
    DatabaseError,
    Timeout,
};

const char* to_string(ErrorCode code);
std::optional<ErrorCode> error_code_from(const std::string& s);

// doc/shared_libraries.md#bus
struct ServiceError : std::runtime_error {
    ServiceError(ErrorCode code, const std::string& message, json details = nullptr)
        : std::runtime_error(message), code(code), details(std::move(details)) {}
    ErrorCode code;
    json details;
};

// doc/interfaces.md#request-envelope
struct RequestEnvelope {
    std::string type;
    int version = MESSAGE_VERSION;
    std::string request_id;
    std::optional<std::string> reply_to;
    json payload = json::object();
};

// doc/interfaces.md#response-envelope
struct ResponseEnvelope {
    std::string type;
    int version = MESSAGE_VERSION;
    std::string request_id;
    bool ok = true;
    json error = nullptr;
    json payload = nullptr;
};

// doc/interfaces.md#event-envelope
struct EventEnvelope {
    std::string type;
    int version = MESSAGE_VERSION;
    std::string event_id;
    std::string source;
    std::int64_t timestamp_ms = 0;
    json payload = json::object();
};

json to_json(const RequestEnvelope& e);
json to_json(const ResponseEnvelope& e);
json to_json(const EventEnvelope& e);
RequestEnvelope parse_request(const json& j);
ResponseEnvelope parse_response(const json& j);
EventEnvelope parse_event(const json& j);

ResponseEnvelope ok_response(const RequestEnvelope& req, json payload);
ResponseEnvelope error_response(const RequestEnvelope& req, const ServiceError& e);
std::string new_uuid();

namespace messages {

// doc/interfaces.md#database_handler
inline constexpr const char* DATABASE_HANDLER_STREAM = "database_handler:requests";
// doc/interfaces.md#camera_reader
inline constexpr const char* CAMERA_READER_STREAM = "camera_reader:requests";
// doc/interfaces.md#image_processor
inline constexpr const char* IMAGE_PROCESSOR_STREAM = "image_processor:requests";

// doc/interfaces.md#save_setting
struct SaveSetting {
    static constexpr const char* TYPE = "save_setting";
    static constexpr const char* STREAM = DATABASE_HANDLER_STREAM;
    static constexpr std::chrono::seconds TIMEOUT = 5s;
    model::Setting setting;
    json payload() const { return setting; }
};

// doc/interfaces.md#read_settings
struct ReadSettings {
    static constexpr const char* TYPE = "read_settings";
    static constexpr const char* STREAM = DATABASE_HANDLER_STREAM;
    static constexpr std::chrono::seconds TIMEOUT = 5s;
    model::Filter filter;
    json payload() const { return json{{"filter", filter}}; }
};

// doc/interfaces.md#camera_status
struct CameraStatus {
    static constexpr const char* TYPE = "camera_status";
    static constexpr const char* STREAM = CAMERA_READER_STREAM;
    static constexpr std::chrono::seconds TIMEOUT = 5s;
    json payload() const { return json::object(); }
};

// doc/interfaces.md#camera_status
struct CameraStatusResponse {
    std::string state;
    std::string camera_source;
    std::optional<std::string> device;
    std::optional<std::string> camera_key;
    std::optional<std::int64_t> width_px;
    std::optional<std::int64_t> height_px;
    std::optional<double> fps;
    std::int64_t frames_published = 0;
};

void to_json(json& j, const CameraStatusResponse& v);
void from_json(const json& j, CameraStatusResponse& v);

// doc/interfaces.md#camera_state_changed
struct CameraStateChanged {
    std::string state;
    std::string message;
    std::optional<std::string> camera_key;
    std::optional<std::int64_t> width_px;
    std::optional<std::int64_t> height_px;
};

void to_json(json& j, const CameraStateChanged& v);
void from_json(const json& j, CameraStateChanged& v);

// doc/interfaces.md#shot_detected
struct ShotDetected {
    std::int64_t hole_id = 0;
    std::int64_t live_paper = 1;
    double angle_turns = 0;
    double radius_mm = 0;
    double confidence = 0;
    std::int64_t frame_seq = 0;
};

void to_json(json& j, const ShotDetected& v);

// doc/interfaces.md#paper_replaced
struct PaperReplaced {
    std::int64_t live_paper = 1;
    std::optional<std::string> previous_paper_staged_file;
};

void to_json(json& j, const PaperReplaced& v);

}  // namespace messages
}  // namespace targetscope
