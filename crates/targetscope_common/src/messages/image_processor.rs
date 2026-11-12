//! Messages of `image_processor` (doc/interfaces.md#image_processor).
//! Source of truth: `schemas/image_processor/`.

use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::Request;
use crate::model::{CalibrationStatus, Corners, TargetType};

pub const STREAM: &str = "image_processor:requests";

/// Requests understood by `image_processor`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum ImageProcessorRequest {
    Ping {},
    LiveStart(LiveStart),
    LiveCalibrate(LiveCalibrate),
    LiveSetCounting(LiveSetCounting),
    LiveUncountHole(LiveUncountHole),
    LiveNewPaper(LiveNewPaper),
    LiveStop(LiveStop),
    LiveCalibrationStatus(LiveCalibrationStatus),
    LensCalibrationStart(LensCalibrationStart),
    LensCalibrationCapture(LensCalibrationCapture),
    LensCalibrationFinish(LensCalibrationFinish),
    LensCalibrationCancel(LensCalibrationCancel),
    LensCalibrationDelete(LensCalibrationDelete),
    LensCalibrationStatus(LensCalibrationStatus),
    CaptureTargetImage(CaptureTargetImage),
    CaptureRawImage(CaptureRawImage),
    FindPaper(FindPaper),
    PrepareTargetPhoto(PrepareTargetPhoto),
    DetectShotsInPhoto(DetectShotsInPhoto),
}

/// Events on `image_processor:events`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum ImageProcessorEvent {
    CalibrationChanged(CalibrationStatus),
    PaperReplaced(PaperReplaced),
    ShotDetected(ShotDetected),
}

/// `live_start` request (doc/interfaces.md#live_start, schemas/image_processor/live_start.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiveStart {
    pub target_type: TargetType,
    pub bullet_diameter_mm: f64,
}

impl Request for LiveStart {
    type Response = CalibrationStatus;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}

/// `live_calibrate` request (doc/interfaces.md#live_calibrate, schemas/image_processor/live_calibrate.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiveCalibrate {
    pub corners: Corners,
}

impl Request for LiveCalibrate {
    type Response = CalibrationStatus;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}

/// `live_set_counting` request (doc/interfaces.md#live_set_counting, schemas/image_processor/live_set_counting.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiveSetCounting {
    pub counting: bool,
}

/// `live_set_counting_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiveSetCountingResponse {}

impl Request for LiveSetCounting {
    type Response = LiveSetCountingResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}

/// `live_uncount_hole` request (doc/interfaces.md#live_uncount_hole, schemas/image_processor/live_uncount_hole.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiveUncountHole {
    pub hole_id: u32,
}

/// `live_uncount_hole_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiveUncountHoleResponse {}

impl Request for LiveUncountHole {
    type Response = LiveUncountHoleResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}

/// `live_new_paper` request (doc/interfaces.md#live_new_paper, schemas/image_processor/live_new_paper.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiveNewPaper {
    pub from_live_paper: u32,
}

/// `live_new_paper_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiveNewPaperResponse {
    pub changed: bool,
    pub live_paper: u32,
    pub previous_paper_staged_file: Option<String>,
}

impl Request for LiveNewPaper {
    type Response = LiveNewPaperResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}

/// `live_stop` request (doc/interfaces.md#live_stop, schemas/image_processor/live_stop.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiveStop {}

/// `live_stop_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiveStopResponse {}

impl Request for LiveStop {
    type Response = LiveStopResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}

/// `live_calibration_status` request (doc/interfaces.md#live_calibration_status, schemas/image_processor/live_calibration_status.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiveCalibrationStatus {}

impl Request for LiveCalibrationStatus {
    type Response = CalibrationStatus;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}

/// `lens_calibration_start` request (doc/interfaces.md#lens_calibration_start, schemas/image_processor/lens_calibration_start.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LensCalibrationStart {}

/// `lens_calibration_start_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LensCalibrationStartResponse {
    // TODO: fields
}

impl Request for LensCalibrationStart {
    type Response = LensCalibrationStartResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}

/// `lens_calibration_capture` request (doc/interfaces.md#lens_calibration_capture, schemas/image_processor/lens_calibration_capture.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LensCalibrationCapture {}

/// `lens_calibration_capture_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LensCalibrationCaptureResponse {
    // TODO: fields
}

impl Request for LensCalibrationCapture {
    type Response = LensCalibrationCaptureResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}

/// `lens_calibration_finish` request (doc/interfaces.md#lens_calibration_finish, schemas/image_processor/lens_calibration_finish.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LensCalibrationFinish {}

/// `lens_calibration_finish_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LensCalibrationFinishResponse {
    // TODO: fields
}

impl Request for LensCalibrationFinish {
    type Response = LensCalibrationFinishResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(60);
}

/// `lens_calibration_cancel` request (doc/interfaces.md#lens_calibration_cancel, schemas/image_processor/lens_calibration_cancel.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LensCalibrationCancel {}

/// `lens_calibration_cancel_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LensCalibrationCancelResponse {}

impl Request for LensCalibrationCancel {
    type Response = LensCalibrationCancelResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}

/// `lens_calibration_delete` request (doc/interfaces.md#lens_calibration_delete, schemas/image_processor/lens_calibration_delete.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LensCalibrationDelete {}

/// `lens_calibration_delete_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LensCalibrationDeleteResponse {}

impl Request for LensCalibrationDelete {
    type Response = LensCalibrationDeleteResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}

/// `lens_calibration_status` request (doc/interfaces.md#lens_calibration_status, schemas/image_processor/lens_calibration_status.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LensCalibrationStatus {}

/// `lens_calibration_status_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LensCalibrationStatusResponse {
    // TODO: fields
}

impl Request for LensCalibrationStatus {
    type Response = LensCalibrationStatusResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}

/// `capture_target_image` request (doc/interfaces.md#capture_target_image, schemas/image_processor/capture_target_image.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaptureTargetImage {}

/// `capture_target_image_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaptureTargetImageResponse {
    pub staged_file: String,
}

impl Request for CaptureTargetImage {
    type Response = CaptureTargetImageResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}

/// `capture_raw_image` request (doc/interfaces.md#capture_raw_image, schemas/image_processor/capture_raw_image.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaptureRawImage {}

/// `capture_raw_image_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaptureRawImageResponse {
    pub staged_file: String,
    pub lens_corrected: bool,
}

impl Request for CaptureRawImage {
    type Response = CaptureRawImageResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}

/// `find_paper` request (doc/interfaces.md#find_paper, schemas/image_processor/find_paper.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FindPaper {
    pub staged_file: String,
}

/// `find_paper_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FindPaperResponse {
    pub corners: Option<Corners>,
    pub width_px: u32,
    pub height_px: u32,
}

impl Request for FindPaper {
    type Response = FindPaperResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(60);
}

/// `prepare_target_photo` request (doc/interfaces.md#prepare_target_photo, schemas/image_processor/prepare_target_photo.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrepareTargetPhoto {
    // TODO: fields
}

/// `prepare_target_photo_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrepareTargetPhotoResponse {
    // TODO: fields
}

impl Request for PrepareTargetPhoto {
    type Response = PrepareTargetPhotoResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(60);
}

/// `detect_shots_in_photo` request (doc/interfaces.md#detect_shots_in_photo, schemas/image_processor/detect_shots_in_photo.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DetectShotsInPhoto {
    // TODO: fields
}

/// `detect_shots_in_photo_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DetectShotsInPhotoResponse {
    // TODO: fields
}

impl Request for DetectShotsInPhoto {
    type Response = DetectShotsInPhotoResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(60);
}

/// `paper_replaced` event (doc/interfaces.md#paper_replaced, schemas/image_processor/paper_replaced.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaperReplaced {
    pub live_paper: u32,
    pub previous_paper_staged_file: Option<String>,
}

/// `shot_detected` event (doc/interfaces.md#shot_detected, schemas/image_processor/shot_detected.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShotDetected {
    pub hole_id: u32,
    pub live_paper: u32,
    pub angle_turns: f64,
    pub radius_mm: f64,
    pub confidence: f32,
    pub frame_seq: u64,
}
