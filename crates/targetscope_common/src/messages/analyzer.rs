//! Messages of `analyzer` (doc/interfaces.md#analyzer).
//! Source of truth: `schemas/analyzer/`.

use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::Request;
use crate::model::{AnalysisResult, Shot, TargetType};

pub const STREAM: &str = "analyzer:requests";

/// Requests understood by `analyzer`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum AnalyzerRequest {
    Ping {},
    AnalyzeShots(AnalyzeShots),
}

/// `analyze_shots` request (doc/interfaces.md#analyze_shots, schemas/analyzer/analyze_shots.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalyzeShots {
    pub shots: Vec<Shot>,
    pub target_type: TargetType,
    pub bullet_diameter_mm: f64,
}

impl Request for AnalyzeShots {
    type Response = AnalysisResult;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}
