// doc/interfaces.md#statistics

use serde::{Deserialize, Serialize};

use super::{request, Ping};
use crate::model::{SessionFilter, StatisticsGroup};

pub const PROGRAM: &str = "statistics";
pub const STREAM: &str = "statistics:requests";

// doc/shared_libraries.md#messages
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum StatisticsRequest {
    Ping(Ping),
    ComputeStatistics(ComputeStatistics),
}

// doc/interfaces.md#compute_statistics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupBy {
    None,
    Shooter,
    Device,
    DeviceCategory,
    Distance,
    SingleHanded,
    Supported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Period {
    Session,
    Day,
    Week,
    Month,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComputeStatistics {
    #[serde(default)]
    pub filter: SessionFilter,
    pub group_by: GroupBy,
    pub period: Period,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComputeStatisticsResponse {
    pub groups: Vec<StatisticsGroup>,
    pub session_count: i64,
}
request!(ComputeStatistics, ComputeStatisticsResponse, "compute_statistics", 90);
