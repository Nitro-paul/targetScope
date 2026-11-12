// doc/interfaces.md#analyzer

use serde::{Deserialize, Serialize};

use super::{request, Ping};
use crate::model::{AnalysisResult, Shot, TargetType};

pub const PROGRAM: &str = "analyzer";
pub const STREAM: &str = "analyzer:requests";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum AnalyzerRequest {
    Ping(Ping),
    AnalyzeShots(AnalyzeShots),
}

// doc/interfaces.md#analyze_shots
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalyzeShots {
    pub shots: Vec<Shot>,
    pub target_type: TargetType,
    pub bullet_diameter_mm: f64,
}
request!(AnalyzeShots, AnalysisResult, "analyze_shots", 5);
