// doc/interfaces.md#image_processor

use serde::{Deserialize, Serialize};

use super::{request, Empty, Ping};
use crate::model::{CalibrationStatus, Corners, TargetType};

pub const PROGRAM: &str = "image_processor";
pub const STREAM: &str = "image_processor:requests";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum ImageProcessorRequest {
    Ping(Ping),
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

// doc/interfaces.md#events-image_processorevents
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum ImageProcessorEvent {
    CalibrationChanged(CalibrationStatus),
    PaperReplaced(PaperReplaced),
    ShotDetected(ShotDetected),
}

// doc/interfaces.md#live_start
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LiveStart {
    pub target_type: TargetType,
    #[serde(deserialize_with = "crate::check::positive")]
    pub bullet_diameter_mm: f64,
}
request!(LiveStart, CalibrationStatus, "live_start", 5);

// doc/interfaces.md#live_calibrate
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LiveCalibrate {
    pub corners: Corners,
}
request!(LiveCalibrate, CalibrationStatus, "live_calibrate", 5);

// doc/interfaces.md#live_set_counting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LiveSetCounting {
    pub counting: bool,
}
request!(LiveSetCounting, Empty, "live_set_counting", 5);

// doc/interfaces.md#live_uncount_hole
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LiveUncountHole {
    pub hole_id: i64,
}
request!(LiveUncountHole, Empty, "live_uncount_hole", 5);

// doc/interfaces.md#live_new_paper
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LiveNewPaper {
    pub from_live_paper: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LiveNewPaperResponse {
    pub changed: bool,
    pub live_paper: i64,
    pub previous_paper_staged_file: Option<String>,
}
request!(LiveNewPaper, LiveNewPaperResponse, "live_new_paper", 5);

// doc/interfaces.md#live_stop
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LiveStop {}
request!(LiveStop, Empty, "live_stop", 5);

// doc/interfaces.md#live_calibration_status
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LiveCalibrationStatus {}
request!(LiveCalibrationStatus, CalibrationStatus, "live_calibration_status", 5);

// doc/interfaces.md#lens-calibration
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LensCalibrationStart {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ViewsResponse {
    pub views: i64,
}
request!(LensCalibrationStart, ViewsResponse, "lens_calibration_start", 5);

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LensCalibrationCapture {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LensCalibrationCaptureResponse {
    pub found: bool,
    pub views: i64,
}
request!(LensCalibrationCapture, LensCalibrationCaptureResponse, "lens_calibration_capture", 5);

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LensCalibrationFinish {}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LensCalibrationFinishResponse {
    pub rms_px: f64,
    pub views: i64,
}
request!(LensCalibrationFinish, LensCalibrationFinishResponse, "lens_calibration_finish", 60);

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LensCalibrationCancel {}
request!(LensCalibrationCancel, Empty, "lens_calibration_cancel", 5);

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LensCalibrationDelete {}
request!(LensCalibrationDelete, Empty, "lens_calibration_delete", 5);

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LensCalibrationStatus {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LensCalibrationStatusResponse {
    pub camera_key: Option<String>,
    pub calibrated: bool,
    pub applied: bool,
    pub rms_px: Option<f64>,
    pub width_px: Option<i64>,
    pub height_px: Option<i64>,
    pub collecting_views: Option<i64>,
}
request!(LensCalibrationStatus, LensCalibrationStatusResponse, "lens_calibration_status", 5);

// doc/interfaces.md#capture_target_image
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureTargetImage {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StagedFileResponse {
    pub staged_file: String,
}
request!(CaptureTargetImage, StagedFileResponse, "capture_target_image", 5);

// doc/interfaces.md#capture_raw_image
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureRawImage {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureRawImageResponse {
    pub staged_file: String,
    pub lens_corrected: bool,
}
request!(CaptureRawImage, CaptureRawImageResponse, "capture_raw_image", 5);

// doc/interfaces.md#find_paper
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FindPaper {
    pub staged_file: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FindPaperResponse {
    pub corners: Option<Corners>,
    pub width_px: i64,
    pub height_px: i64,
}
request!(FindPaper, FindPaperResponse, "find_paper", 60);

// doc/interfaces.md#prepare_target_photo
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrepareTargetPhoto {
    pub staged_file: String,
    pub corners: Corners,
    #[serde(deserialize_with = "crate::check::positive")]
    pub paper_width_mm: f64,
    #[serde(deserialize_with = "crate::check::positive")]
    pub paper_height_mm: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RingProposal {
    pub center_x_mm: Option<f64>,
    pub center_y_mm: Option<f64>,
    pub inner_white_radius_mm: Option<f64>,
    pub black_radius_mm: Option<f64>,
    pub outer_radius_mm: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrepareTargetPhotoResponse {
    pub staged_file: String,
    pub px_per_mm: f64,
    pub proposal: RingProposal,
}
request!(PrepareTargetPhoto, PrepareTargetPhotoResponse, "prepare_target_photo", 60);

// doc/interfaces.md#detect_shots_in_photo
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DetectShotsInPhoto {
    pub staged_file: String,
    pub target_type: TargetType,
    #[serde(deserialize_with = "crate::check::positive")]
    pub bullet_diameter_mm: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub corners: Option<Corners>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DetectedShot {
    #[serde(deserialize_with = "crate::check::angle_turns")]
    pub angle_turns: f64,
    #[serde(deserialize_with = "crate::check::non_negative")]
    pub radius_mm: f64,
    #[serde(deserialize_with = "crate::check::unit")]
    pub confidence: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DetectShotsInPhotoResponse {
    pub shots: Vec<DetectedShot>,
    pub staged_file: String,
    pub warnings: Vec<String>,
}
request!(DetectShotsInPhoto, DetectShotsInPhotoResponse, "detect_shots_in_photo", 60);

// doc/interfaces.md#paper_replaced
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PaperReplaced {
    pub live_paper: i64,
    pub previous_paper_staged_file: Option<String>,
}

// doc/interfaces.md#shot_detected
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShotDetected {
    pub hole_id: i64,
    pub live_paper: i64,
    #[serde(deserialize_with = "crate::check::angle_turns")]
    pub angle_turns: f64,
    #[serde(deserialize_with = "crate::check::non_negative")]
    pub radius_mm: f64,
    #[serde(deserialize_with = "crate::check::unit")]
    pub confidence: f64,
    pub frame_seq: i64,
}
