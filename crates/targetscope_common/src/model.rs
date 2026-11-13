// doc/interfaces.md#shared-types

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// doc/interfaces.md#session
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub id: i64,
    pub uuid: Uuid,
    pub saved_at: i64,
    pub score: i64,
    pub grouping_score: i64,
    #[serde(default, deserialize_with = "crate::check::opt_count")]
    pub distance_m: Option<i64>,
    pub target_type_id: i64,
    pub shooter_id: Option<i64>,
    pub device_id: i64,
    pub single_handed: bool,
    pub supported: bool,
    #[serde(deserialize_with = "crate::check::number")]
    pub shot_count: i64,
    #[serde(default, deserialize_with = "crate::check::opt_count")]
    pub duration_s: Option<i64>,
    #[serde(default, deserialize_with = "crate::check::notes")]
    pub notes: Option<String>,
    #[serde(deserialize_with = "crate::check::number")]
    pub paper_count: i64,
    #[serde(default)]
    pub images: Vec<PaperImage>,
    #[serde(default)]
    pub shots: Vec<SessionShot>,
}

// doc/interfaces.md#create_session
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NewSession {
    pub score: i64,
    pub grouping_score: i64,
    #[serde(default, deserialize_with = "crate::check::opt_count")]
    pub distance_m: Option<i64>,
    pub target_type_id: i64,
    pub shooter_id: Option<i64>,
    pub device_id: i64,
    pub single_handed: bool,
    pub supported: bool,
    #[serde(deserialize_with = "crate::check::number")]
    pub shot_count: i64,
    #[serde(default, deserialize_with = "crate::check::opt_count")]
    pub duration_s: Option<i64>,
    #[serde(default, deserialize_with = "crate::check::notes")]
    pub notes: Option<String>,
    #[serde(deserialize_with = "crate::check::number")]
    pub paper_count: i64,
    #[serde(deserialize_with = "crate::check::non_empty")]
    pub shots: Vec<SessionShot>,
    #[serde(default)]
    pub images: Vec<StagedPaperImage>,
}

// doc/interfaces.md#import_session
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "serde_json::Map<String, serde_json::Value>")]
pub struct ImportedSession {
    pub uuid: Uuid,
    pub saved_at: i64,
    #[serde(flatten)]
    pub session: NewSession,
}

// doc/shared_libraries.md#value-checks
impl TryFrom<serde_json::Map<String, serde_json::Value>> for ImportedSession {
    type Error = String;

    fn try_from(mut m: serde_json::Map<String, serde_json::Value>) -> Result<Self, String> {
        let mut take = |field: &str| m.remove(field).ok_or_else(|| format!("missing field `{field}`"));
        let uuid = serde_json::from_value(take("uuid")?).map_err(|e| e.to_string())?;
        let saved_at = serde_json::from_value(take("saved_at")?).map_err(|e| e.to_string())?;
        let session = serde_json::from_value(serde_json::Value::Object(m)).map_err(|e| e.to_string())?;
        Ok(ImportedSession { uuid, saved_at, session })
    }
}

// doc/interfaces.md#session
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PaperImage {
    #[serde(deserialize_with = "crate::check::number")]
    pub paper_no: i64,
    pub image_path: String,
}

// doc/interfaces.md#create_session
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StagedPaperImage {
    #[serde(deserialize_with = "crate::check::number")]
    pub paper_no: i64,
    pub staged_file: String,
}

// doc/interfaces.md#shot
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Shot {
    #[serde(deserialize_with = "crate::check::angle_turns")]
    pub angle_turns: f64,
    #[serde(deserialize_with = "crate::check::non_negative")]
    pub radius_mm: f64,
}

impl Shot {
    // doc/interfaces.md#shot
    pub fn is_valid(&self) -> bool {
        self.angle_turns.is_finite() && self.radius_mm.is_finite() && (0.0..1.0).contains(&self.angle_turns) && self.radius_mm >= 0.0
    }
}

// doc/interfaces.md#sessionshot
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionShot {
    #[serde(deserialize_with = "crate::check::angle_turns")]
    pub angle_turns: f64,
    #[serde(deserialize_with = "crate::check::non_negative")]
    pub radius_mm: f64,
    #[serde(deserialize_with = "crate::check::number")]
    pub paper_no: i64,
}

impl SessionShot {
    pub fn shot(&self) -> Shot {
        Shot { angle_turns: self.angle_turns, radius_mm: self.radius_mm }
    }
}

// doc/interfaces.md#shooter
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Shooter {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uuid: Option<Uuid>,
    #[serde(deserialize_with = "crate::check::name")]
    pub name: String,
    #[serde(default)]
    pub retired: bool,
}

// doc/interfaces.md#device
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Device {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uuid: Option<Uuid>,
    #[serde(deserialize_with = "crate::check::name")]
    pub name: String,
    pub category: Option<String>,
    pub description: Option<String>,
    pub caliber: Option<String>,
    #[serde(deserialize_with = "crate::check::positive")]
    pub bullet_diameter_mm: f64,
    #[serde(default)]
    pub image_path: Option<String>,
    #[serde(default, deserialize_with = "crate::check::count")]
    pub shot_count: i64,
    #[serde(default)]
    pub locked: bool,
    #[serde(default)]
    pub retired: bool,
}

// doc/interfaces.md#targettype
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetType {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uuid: Option<Uuid>,
    #[serde(deserialize_with = "crate::check::name")]
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
    #[serde(default)]
    pub rings: Vec<Ring>,
    #[serde(default)]
    pub locked: bool,
    #[serde(default)]
    pub retired: bool,
}

// doc/interfaces.md#ring
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ring {
    pub score: i64,
    pub radius_mm: f64,
}

// doc/interfaces.md#setting
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Setting {
    #[serde(deserialize_with = "crate::check::name")]
    pub name: String,
    pub display_name: String,
    pub active: bool,
    pub value: String,
    #[serde(default)]
    pub optional: bool,
    #[serde(default)]
    pub default_value: String,
    #[serde(default)]
    pub default_active: bool,
    #[serde(default)]
    pub value_rule: String,
}

// doc/interfaces.md#filter
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Filter {
    #[serde(default)]
    pub conditions: Vec<Condition>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order_by: Option<OrderBy>,
    #[serde(default, deserialize_with = "crate::check::opt_count", skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(default, deserialize_with = "crate::check::opt_count", skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
}

impl Filter {
    pub fn eq(field: &str, value: impl Into<serde_json::Value>) -> Self {
        Self::with(field, FilterOp::Eq, value)
    }

    pub fn with(field: &str, op: FilterOp, value: impl Into<serde_json::Value>) -> Self {
        Filter { conditions: vec![Condition { field: field.to_string(), op, value: value.into() }], ..Default::default() }
    }
}

// doc/interfaces.md#filter
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Condition {
    pub field: String,
    pub op: FilterOp,
    pub value: serde_json::Value,
}

// doc/interfaces.md#filter
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

// doc/interfaces.md#filter
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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

// doc/interfaces.md#sessionfilter
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionFilter {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_ids: Option<Vec<i64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shooter_ids: Option<Vec<i64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_ids: Option<Vec<i64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_categories: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub calibers: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_type_ids: Option<Vec<i64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub distance_m: Option<IntRange>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub saved_at: Option<TimeRange>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub single_handed: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supported: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shot_count: Option<IntRange>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include_retired: Option<bool>,
}

// doc/interfaces.md#sessionfilter
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntRange {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<i64>,
}

// doc/interfaces.md#sessionfilter
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimeRange {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<i64>,
}

// doc/interfaces.md#corners
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Corners {
    pub top_left: PixelPoint,
    pub top_right: PixelPoint,
    pub bottom_right: PixelPoint,
    pub bottom_left: PixelPoint,
}

// doc/interfaces.md#corners
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PixelPoint {
    pub x: f64,
    pub y: f64,
}

// doc/interfaces.md#analysisresult
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OffsetMm {
    pub x_mm: f64,
    pub y_mm: f64,
}

// doc/interfaces.md#analysisresult
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnalysisResult {
    pub shot_scores: Vec<i64>,
    pub score: i64,
    pub grouping_score: i64,
    pub grouping_shift: OffsetMm,
    pub mean_point: Option<OffsetMm>,
}

// doc/interfaces.md#currentsession
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionState {
    Idle,
    Setup,
    Running,
    Paused,
    Review,
    Recovered,
}

// doc/interfaces.md#currentsession
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CurrentSession {
    pub state: SessionState,
    pub recovered_from: Option<SessionState>,
    pub setup: SessionSetup,
    pub shots: Vec<CurrentShot>,
    #[serde(deserialize_with = "crate::check::number")]
    pub paper_no: i64,
    pub paper_images: Vec<StagedPaperImage>,
    pub notes: Option<String>,
    pub analysis: Option<AnalysisResult>,
    pub analysis_outdated: bool,
    pub started_at: Option<i64>,
    #[serde(deserialize_with = "crate::check::count")]
    pub duration_s: i64,
    pub calibration: Option<CalibrationStatus>,
}

// doc/interfaces.md#sessionsetup
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputMode {
    Camera,
    Manual,
    Photo,
}

// doc/interfaces.md#sessionsetup
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionSetup {
    pub input_mode: InputMode,
    pub shooter_id: Option<i64>,
    pub device_id: Option<i64>,
    #[serde(default, deserialize_with = "crate::check::opt_count")]
    pub distance_m: Option<i64>,
    pub target_type_id: Option<i64>,
    pub single_handed: bool,
    pub supported: bool,
}

// doc/interfaces.md#currentshot
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CurrentShot {
    #[serde(deserialize_with = "crate::check::angle_turns")]
    pub angle_turns: f64,
    #[serde(deserialize_with = "crate::check::non_negative")]
    pub radius_mm: f64,
    pub origin: InputMode,
    #[serde(default, deserialize_with = "crate::check::opt_unit")]
    pub confidence: Option<f64>,
    pub hole_id: Option<i64>,
    #[serde(deserialize_with = "crate::check::number")]
    pub paper_no: i64,
}

impl CurrentShot {
    pub fn shot(&self) -> Shot {
        Shot { angle_turns: self.angle_turns, radius_mm: self.radius_mm }
    }
}

// doc/interfaces.md#calibrationstatus
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CalibrationState {
    NotCalibrated,
    Calibrated,
    Lost,
}

// doc/interfaces.md#calibrationstatus
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CalibrationMethod {
    Auto,
    Manual,
}

// doc/interfaces.md#calibrationstatus
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CalibrationStatus {
    pub state: CalibrationState,
    pub method: Option<CalibrationMethod>,
    pub target_type_id: Option<i64>,
    pub corners: Option<Corners>,
    pub px_per_mm: Option<f64>,
    pub message: Option<String>,
    pub updated_ms: i64,
}

// doc/interfaces.md#cameracontrol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlKind {
    Integer,
    Boolean,
    Menu,
    Button,
}

// doc/interfaces.md#cameracontrol
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CameraControl {
    pub id: i64,
    pub name: String,
    pub kind: ControlKind,
    pub min: i64,
    pub max: i64,
    pub step: i64,
    pub default: i64,
    pub value: i64,
    #[serde(default)]
    pub menu: Vec<MenuEntry>,
    pub inactive: bool,
}

// doc/interfaces.md#cameracontrol
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MenuEntry {
    pub value: i64,
    pub label: String,
}

// doc/interfaces.md#statisticsgroup
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StatisticsGroup {
    pub target_type_id: i64,
    pub group_key: Option<serde_json::Value>,
    pub summary: StatisticsValues,
    pub trend: Vec<TrendEntry>,
    pub shots: Vec<Shot>,
}

// doc/interfaces.md#statisticsgroup
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrendEntry {
    pub period_start: i64,
    pub values: StatisticsValues,
}

// doc/interfaces.md#statisticsvalues
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
