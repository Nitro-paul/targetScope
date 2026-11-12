// doc/programs/exporter.md#json
// doc/shared_libraries.md#export-file

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::model::SessionShot;

pub const FORMAT: &str = "targetscope-export";
pub const FORMAT_VERSION: u32 = 1;
pub const JSON_ENTRY: &str = "export.json";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportFile {
    pub format: String,
    pub format_version: u32,
    pub exported_at: String,
    pub shooters: Vec<ExportShooter>,
    pub target_types: Vec<ExportTargetType>,
    pub devices: Vec<ExportDevice>,
    pub sessions: Vec<ExportSession>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportShooter {
    pub uuid: Uuid,
    pub name: String,
    pub retired: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportTargetType {
    pub uuid: Uuid,
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
    pub retired: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportDevice {
    pub uuid: Uuid,
    pub name: String,
    pub category: Option<String>,
    pub description: Option<String>,
    pub caliber: Option<String>,
    pub bullet_diameter_mm: f64,
    pub shot_count: i64,
    pub retired: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_entry: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportSession {
    pub uuid: Uuid,
    pub saved_at: i64,
    pub score: i64,
    pub grouping_score: i64,
    pub distance_m: Option<i64>,
    pub shooter_uuid: Option<Uuid>,
    pub device_uuid: Uuid,
    pub target_type_uuid: Uuid,
    pub single_handed: bool,
    pub supported: bool,
    pub shot_count: i64,
    pub duration_s: Option<i64>,
    pub notes: Option<String>,
    pub paper_count: i64,
    pub shots: Vec<SessionShot>,
    pub shot_scores: Vec<i64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub images: Vec<ExportImage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportImage {
    pub paper_no: i64,
    pub image_entry: String,
}

// doc/programs/exporter.md#zip-with-images
pub fn session_image_entry(uuid: &Uuid, paper_no: i64, ext: &str) -> String {
    format!("images/sessions/{uuid}_{paper_no}.{ext}")
}

pub fn device_image_entry(uuid: &Uuid, ext: &str) -> String {
    format!("images/devices/{uuid}.{ext}")
}
