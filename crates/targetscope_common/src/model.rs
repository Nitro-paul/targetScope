//! Model types shared by all programs (doc/interfaces.md#shared-types).
//! Field names are the JSON names of the schemas in `schemas/common/`.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A saved session record (doc/interfaces.md#session).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Session {
    pub id: i64,
    pub uuid: Uuid,
    pub saved_at: i64,
    pub score: i64,
    pub grouping_score: i64,
    pub distance_m: Option<i64>,
    pub target_type_id: i64,
    pub shooter_id: Option<i64>,
    pub device_id: i64,
    pub single_handed: bool,
    pub supported: bool,
    pub shot_count: i64,
    pub duration_s: Option<i64>,
    pub notes: Option<String>,
    pub paper_count: i64,
    pub images: Vec<PaperImage>,
    pub shots: Vec<SessionShot>,
}

/// Payload of `create_session`: all Session fields except id, uuid, saved_at; images as staged files.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewSession {
    // TODO: fields (doc/interfaces.md#create_session)
}

/// Stored image of one target paper of a session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaperImage {
    pub paper_no: i64,
    pub image_path: String,
}

/// A shot in polar coordinates around the ring center (doc/interfaces.md#shot).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Shot {
    pub angle_turns: f64,
    pub radius_mm: f64,
}

/// A shot of a saved session, with the paper it was on.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SessionShot {
    #[serde(flatten)]
    pub shot: Shot,
    pub paper_no: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Shooter {
    pub id: Option<i64>,
    pub uuid: Option<Uuid>,
    pub name: String,
    pub retired: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Device {
    pub id: Option<i64>,
    pub uuid: Option<Uuid>,
    pub name: String,
    pub category: Option<String>,
    pub description: Option<String>,
    pub caliber: Option<String>,
    pub bullet_diameter_mm: f64,
    pub image_path: Option<String>,
    pub shot_count: i64,
    pub locked: bool,
    pub retired: bool,
}

/// Paper target with its three ring anchors (doc/interfaces.md#targettype).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TargetType {
    pub id: Option<i64>,
    pub uuid: Option<Uuid>,
    pub name: String,
    pub paper_width_mm: f64,
    pub paper_height_mm: f64,
    pub center_x_mm: f64,
    pub center_y_mm: f64,
    pub inner_white_radius_mm: f64,
    pub inner_white_score: i64,
    pub black_radius_mm: f64,
    pub black_score: i64,
    pub outer_radius_mm: f64,
    pub outer_score: i64,
    /// Output only: generated with `targets::generate_rings`.
    #[serde(default)]
    pub rings: Vec<Ring>,
    #[serde(default)]
    pub locked: bool,
    pub retired: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Ring {
    pub score: i64,
    pub radius_mm: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Setting {
    pub name: String,
    pub display_name: String,
    pub active: bool,
    pub value: String,
    pub optional: bool,
    pub default_value: String,
    pub default_active: bool,
    pub value_rule: String,
}

/// Generic filter of the read requests for shooters, devices, target types and settings.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Filter {
    #[serde(default)]
    pub conditions: Vec<Condition>,
    pub order_by: Option<OrderBy>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Condition {
    pub field: String,
    pub op: FilterOp,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Like,
    In,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrderBy {
    pub field: String,
    pub direction: Direction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Asc,
    Desc,
}

/// Selects saved sessions (doc/interfaces.md#sessionfilter). Every field is optional.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SessionFilter {
    pub session_ids: Option<Vec<i64>>,
    pub shooter_ids: Option<Vec<i64>>,
    pub device_ids: Option<Vec<i64>>,
    pub device_categories: Option<Vec<String>>,
    pub calibers: Option<Vec<String>>,
    pub target_type_ids: Option<Vec<i64>>,
    pub distance_m: Option<IntRange>,
    pub saved_at: Option<TimeRange>,
    pub single_handed: Option<bool>,
    pub supported: Option<bool>,
    pub shot_count: Option<IntRange>,
    pub include_retired: Option<bool>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct IntRange {
    pub min: Option<i64>,
    pub max: Option<i64>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct TimeRange {
    pub from: Option<i64>,
    pub to: Option<i64>,
}

/// Paper corners in image pixels (origin top-left, x right, y down).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Corners {
    pub top_left: PixelPoint,
    pub top_right: PixelPoint,
    pub bottom_right: PixelPoint,
    pub bottom_left: PixelPoint,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PixelPoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct OffsetMm {
    pub x_mm: f64,
    pub y_mm: f64,
}

/// Result of `analyze_shots` (doc/interfaces.md#analysisresult).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub shot_scores: Vec<i64>,
    pub score: i64,
    pub grouping_score: i64,
    pub grouping_shift: OffsetMm,
    pub mean_point: Option<OffsetMm>,
}

/// State of the current session in session_manager (doc/interfaces.md#currentsession).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CurrentSession {
    pub state: String, // TODO: enum of the states in doc/programs/session_manager.md#state-machine
    pub recovered_from: Option<String>,
    pub setup: SessionSetup,
    pub shots: Vec<CurrentShot>,
    pub paper_no: i64,
    pub paper_images: Vec<StagedPaperImage>,
    pub notes: Option<String>,
    pub analysis: Option<AnalysisResult>,
    pub analysis_outdated: bool,
    pub started_at: Option<i64>,
    pub duration_s: i64,
    pub calibration: Option<CalibrationStatus>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StagedPaperImage {
    pub paper_no: i64,
    pub staged_file: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionSetup {
    pub input_mode: String, // TODO: enum camera | manual | photo
    pub shooter_id: Option<i64>,
    pub device_id: Option<i64>,
    pub distance_m: Option<i64>,
    pub target_type_id: Option<i64>,
    pub single_handed: bool,
    pub supported: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CurrentShot {
    pub angle_turns: f64,
    pub radius_mm: f64,
    pub origin: String, // TODO: enum (doc/interfaces.md#currentshot)
    pub confidence: Option<f32>,
    pub hole_id: Option<i64>,
    pub paper_no: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalibrationStatus {
    pub state: String,          // TODO: enum (doc/interfaces.md#calibrationstatus)
    pub method: Option<String>, // TODO: enum auto | manual
    pub target_type_id: Option<i64>,
    pub corners: Option<Corners>,
    pub px_per_mm: Option<f64>,
    pub message: Option<String>,
    pub updated_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CameraControl {
    pub id: i64,
    pub name: String,
    pub kind: String, // TODO: enum (doc/interfaces.md#cameracontrol)
    pub min: i64,
    pub max: i64,
    pub step: i64,
    pub default: i64,
    pub value: i64,
    #[serde(default)]
    pub menu: Vec<MenuEntry>,
    pub inactive: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MenuEntry {
    pub value: i64,
    pub label: String,
}

/// One group of `compute_statistics` (doc/interfaces.md#statisticsgroup).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StatisticsGroup {
    pub target_type_id: i64,
    pub group_key: Option<serde_json::Value>,
    pub summary: StatisticsValues,
    pub trend: Vec<TrendEntry>,
    pub shots: Vec<Shot>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrendEntry {
    pub period_start: i64,
    pub values: StatisticsValues,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StatisticsValues {
    pub session_count: i64,
    pub shot_count: i64,
    pub score_avg: f64,
    pub score_min: i64,
    pub score_max: i64,
    pub score_per_shot_avg: f64,
    pub grouping_score_per_shot_avg: f64,
    pub extreme_spread_mm_avg: f64,
    pub mean_radius_mm_avg: f64,
    pub mean_point_avg: OffsetMm,
}
