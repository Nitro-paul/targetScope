//! Messages of `statistics` (doc/interfaces.md#statistics).
//! Source of truth: `schemas/statistics/`.

use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::Request;

pub const STREAM: &str = "statistics:requests";

/// Requests understood by `statistics`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum StatisticsRequest {
    Ping {},
    ComputeStatistics(ComputeStatistics),
}

/// `compute_statistics` request (doc/interfaces.md#compute_statistics, schemas/statistics/compute_statistics.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComputeStatistics {
    // TODO: fields
}

/// `compute_statistics_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComputeStatisticsResponse {
    // TODO: fields
}

impl Request for ComputeStatistics {
    type Response = ComputeStatisticsResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(90);
}
