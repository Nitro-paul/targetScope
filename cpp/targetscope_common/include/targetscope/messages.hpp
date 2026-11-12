// Messages of the C++ programs (camera_reader, image_processor) and the requests they send.
// Generated skeleton from doc/interfaces.md; the source of truth is schemas/.
#pragma once

#include <chrono>
#include <cstdint>
#include <optional>
#include <string>
#include <vector>

#include <nlohmann/json.hpp>

#include "targetscope/model.hpp"

namespace targetscope::messages {

using namespace std::chrono_literals;

// ---- database_handler (doc/interfaces.md#database_handler)
inline constexpr const char* DATABASE_HANDLER_STREAM = "database_handler:requests";

// save_setting_response
struct SaveSettingResponse {
    std::string name{};
};
// save_setting request
struct SaveSetting {
    static constexpr const char* TYPE = "save_setting";
    static constexpr const char* STREAM = DATABASE_HANDLER_STREAM;
    static constexpr std::chrono::seconds TIMEOUT = 5s;
    using Response = SaveSettingResponse;
    // TODO: fields
};

// read_settings_response
struct ReadSettingsResponse {
    std::vector<model::Setting> settings{};
};
// read_settings request
struct ReadSettings {
    static constexpr const char* TYPE = "read_settings";
    static constexpr const char* STREAM = DATABASE_HANDLER_STREAM;
    static constexpr std::chrono::seconds TIMEOUT = 5s;
    using Response = ReadSettingsResponse;
    // TODO: fields
};

// ---- camera_reader (doc/interfaces.md#camera_reader)
inline constexpr const char* CAMERA_READER_STREAM = "camera_reader:requests";

// camera_status_response
struct CameraStatusResponse {
    // TODO: fields
};
// camera_status request
struct CameraStatus {
    static constexpr const char* TYPE = "camera_status";
    static constexpr const char* STREAM = CAMERA_READER_STREAM;
    static constexpr std::chrono::seconds TIMEOUT = 5s;
    using Response = CameraStatusResponse;
};

// camera_list_devices_response
struct CameraListDevicesResponse {
    // TODO: fields
};
// camera_list_devices request
struct CameraListDevices {
    static constexpr const char* TYPE = "camera_list_devices";
    static constexpr const char* STREAM = CAMERA_READER_STREAM;
    static constexpr std::chrono::seconds TIMEOUT = 5s;
    using Response = CameraListDevicesResponse;
};

// camera_list_controls_response
struct CameraListControlsResponse {
    std::vector<model::CameraControl> controls{};
};
// camera_list_controls request
struct CameraListControls {
    static constexpr const char* TYPE = "camera_list_controls";
    static constexpr const char* STREAM = CAMERA_READER_STREAM;
    static constexpr std::chrono::seconds TIMEOUT = 5s;
    using Response = CameraListControlsResponse;
};

// camera_set_control_response
struct CameraSetControlResponse {
    std::vector<model::CameraControl> controls{};
};
// camera_set_control request
struct CameraSetControl {
    static constexpr const char* TYPE = "camera_set_control";
    static constexpr const char* STREAM = CAMERA_READER_STREAM;
    static constexpr std::chrono::seconds TIMEOUT = 5s;
    using Response = CameraSetControlResponse;
    std::int64_t id{};
    std::int64_t value{};
};

// camera_state_changed event
struct CameraStateChanged {
    static constexpr const char* TYPE = "camera_state_changed";
    // TODO: fields
};

// ---- image_processor (doc/interfaces.md#image_processor)
inline constexpr const char* IMAGE_PROCESSOR_STREAM = "image_processor:requests";

// live_start request
struct LiveStart {
    static constexpr const char* TYPE = "live_start";
    static constexpr const char* STREAM = IMAGE_PROCESSOR_STREAM;
    static constexpr std::chrono::seconds TIMEOUT = 5s;
    using Response = model::CalibrationStatus;
    model::TargetType target_type{};
    double bullet_diameter_mm{};
};

// live_calibrate request
struct LiveCalibrate {
    static constexpr const char* TYPE = "live_calibrate";
    static constexpr const char* STREAM = IMAGE_PROCESSOR_STREAM;
    static constexpr std::chrono::seconds TIMEOUT = 5s;
    using Response = model::CalibrationStatus;
    model::Corners corners{};
};

// live_set_counting_response
struct LiveSetCountingResponse {
};
// live_set_counting request
struct LiveSetCounting {
    static constexpr const char* TYPE = "live_set_counting";
    static constexpr const char* STREAM = IMAGE_PROCESSOR_STREAM;
    static constexpr std::chrono::seconds TIMEOUT = 5s;
    using Response = LiveSetCountingResponse;
    bool counting{};
};

// live_uncount_hole_response
struct LiveUncountHoleResponse {
};
// live_uncount_hole request
struct LiveUncountHole {
    static constexpr const char* TYPE = "live_uncount_hole";
    static constexpr const char* STREAM = IMAGE_PROCESSOR_STREAM;
    static constexpr std::chrono::seconds TIMEOUT = 5s;
    using Response = LiveUncountHoleResponse;
    std::uint32_t hole_id{};
};

// live_new_paper_response
struct LiveNewPaperResponse {
    bool changed{};
    std::uint32_t live_paper{};
    std::optional<std::string> previous_paper_staged_file{};
};
// live_new_paper request
struct LiveNewPaper {
    static constexpr const char* TYPE = "live_new_paper";
    static constexpr const char* STREAM = IMAGE_PROCESSOR_STREAM;
    static constexpr std::chrono::seconds TIMEOUT = 5s;
    using Response = LiveNewPaperResponse;
    std::uint32_t from_live_paper{};
};

// live_stop_response
struct LiveStopResponse {
};
// live_stop request
struct LiveStop {
    static constexpr const char* TYPE = "live_stop";
    static constexpr const char* STREAM = IMAGE_PROCESSOR_STREAM;
    static constexpr std::chrono::seconds TIMEOUT = 5s;
    using Response = LiveStopResponse;
};

// live_calibration_status request
struct LiveCalibrationStatus {
    static constexpr const char* TYPE = "live_calibration_status";
    static constexpr const char* STREAM = IMAGE_PROCESSOR_STREAM;
    static constexpr std::chrono::seconds TIMEOUT = 5s;
    using Response = model::CalibrationStatus;
};

// lens_calibration_start_response
struct LensCalibrationStartResponse {
    // TODO: fields
};
// lens_calibration_start request
struct LensCalibrationStart {
    static constexpr const char* TYPE = "lens_calibration_start";
    static constexpr const char* STREAM = IMAGE_PROCESSOR_STREAM;
    static constexpr std::chrono::seconds TIMEOUT = 5s;
    using Response = LensCalibrationStartResponse;
};

// lens_calibration_capture_response
struct LensCalibrationCaptureResponse {
    // TODO: fields
};
// lens_calibration_capture request
struct LensCalibrationCapture {
    static constexpr const char* TYPE = "lens_calibration_capture";
    static constexpr const char* STREAM = IMAGE_PROCESSOR_STREAM;
    static constexpr std::chrono::seconds TIMEOUT = 5s;
    using Response = LensCalibrationCaptureResponse;
};

// lens_calibration_finish_response
struct LensCalibrationFinishResponse {
    // TODO: fields
};
// lens_calibration_finish request
struct LensCalibrationFinish {
    static constexpr const char* TYPE = "lens_calibration_finish";
    static constexpr const char* STREAM = IMAGE_PROCESSOR_STREAM;
    static constexpr std::chrono::seconds TIMEOUT = 60s;
    using Response = LensCalibrationFinishResponse;
};

// lens_calibration_cancel_response
struct LensCalibrationCancelResponse {
};
// lens_calibration_cancel request
struct LensCalibrationCancel {
    static constexpr const char* TYPE = "lens_calibration_cancel";
    static constexpr const char* STREAM = IMAGE_PROCESSOR_STREAM;
    static constexpr std::chrono::seconds TIMEOUT = 5s;
    using Response = LensCalibrationCancelResponse;
};

// lens_calibration_delete_response
struct LensCalibrationDeleteResponse {
};
// lens_calibration_delete request
struct LensCalibrationDelete {
    static constexpr const char* TYPE = "lens_calibration_delete";
    static constexpr const char* STREAM = IMAGE_PROCESSOR_STREAM;
    static constexpr std::chrono::seconds TIMEOUT = 5s;
    using Response = LensCalibrationDeleteResponse;
};

// lens_calibration_status_response
struct LensCalibrationStatusResponse {
    // TODO: fields
};
// lens_calibration_status request
struct LensCalibrationStatus {
    static constexpr const char* TYPE = "lens_calibration_status";
    static constexpr const char* STREAM = IMAGE_PROCESSOR_STREAM;
    static constexpr std::chrono::seconds TIMEOUT = 5s;
    using Response = LensCalibrationStatusResponse;
};

// capture_target_image_response
struct CaptureTargetImageResponse {
    std::string staged_file{};
};
// capture_target_image request
struct CaptureTargetImage {
    static constexpr const char* TYPE = "capture_target_image";
    static constexpr const char* STREAM = IMAGE_PROCESSOR_STREAM;
    static constexpr std::chrono::seconds TIMEOUT = 5s;
    using Response = CaptureTargetImageResponse;
};

// capture_raw_image_response
struct CaptureRawImageResponse {
    std::string staged_file{};
    bool lens_corrected{};
};
// capture_raw_image request
struct CaptureRawImage {
    static constexpr const char* TYPE = "capture_raw_image";
    static constexpr const char* STREAM = IMAGE_PROCESSOR_STREAM;
    static constexpr std::chrono::seconds TIMEOUT = 5s;
    using Response = CaptureRawImageResponse;
};

// find_paper_response
struct FindPaperResponse {
    std::optional<model::Corners> corners{};
    std::uint32_t width_px{};
    std::uint32_t height_px{};
};
// find_paper request
struct FindPaper {
    static constexpr const char* TYPE = "find_paper";
    static constexpr const char* STREAM = IMAGE_PROCESSOR_STREAM;
    static constexpr std::chrono::seconds TIMEOUT = 60s;
    using Response = FindPaperResponse;
    std::string staged_file{};
};

// prepare_target_photo_response
struct PrepareTargetPhotoResponse {
    // TODO: fields
};
// prepare_target_photo request
struct PrepareTargetPhoto {
    static constexpr const char* TYPE = "prepare_target_photo";
    static constexpr const char* STREAM = IMAGE_PROCESSOR_STREAM;
    static constexpr std::chrono::seconds TIMEOUT = 60s;
    using Response = PrepareTargetPhotoResponse;
    // TODO: fields
};

// detect_shots_in_photo_response
struct DetectShotsInPhotoResponse {
    // TODO: fields
};
// detect_shots_in_photo request
struct DetectShotsInPhoto {
    static constexpr const char* TYPE = "detect_shots_in_photo";
    static constexpr const char* STREAM = IMAGE_PROCESSOR_STREAM;
    static constexpr std::chrono::seconds TIMEOUT = 60s;
    using Response = DetectShotsInPhotoResponse;
    // TODO: fields
};

// paper_replaced event
struct PaperReplaced {
    static constexpr const char* TYPE = "paper_replaced";
    std::uint32_t live_paper{};
    std::optional<std::string> previous_paper_staged_file{};
};

// shot_detected event
struct ShotDetected {
    static constexpr const char* TYPE = "shot_detected";
    std::uint32_t hole_id{};
    std::uint32_t live_paper{};
    double angle_turns{};
    double radius_mm{};
    float confidence{};
    std::uint64_t frame_seq{};
};

// TODO: to_json / from_json for every struct above (NLOHMANN_DEFINE_TYPE_NON_INTRUSIVE)

}  // namespace targetscope::messages

namespace targetscope {

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
    Timeout,  // set by the sender only
};

const char* to_string(ErrorCode code);

inline constexpr int MESSAGE_VERSION = 1;

}  // namespace targetscope
