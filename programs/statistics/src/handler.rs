// doc/programs/statistics.md#calculation-flow-compute_statistics

use std::collections::{BTreeMap, BTreeSet, HashMap};

use async_trait::async_trait;
use serde_json::Value;
use targetscope_common::bus::{Bus, RequestHandler};
use targetscope_common::messages::statistics::{ComputeStatistics, ComputeStatisticsResponse, GroupBy, StatisticsRequest};
use targetscope_common::messages::ServiceError;
use targetscope_common::model::{Session, StatisticsGroup, TrendEntry};
use uuid::Uuid;

use crate::calc::{aggregate, distribution_shots, summarize, SessionSummary};
use crate::grouping::group_sessions;
use crate::periods::period_start;
use crate::reading::{read_all_sessions, read_devices, read_target_types};

pub struct StatisticsHandler {
    pub bus: Bus,
}

#[async_trait]
impl RequestHandler<StatisticsRequest> for StatisticsHandler {
    async fn handle(&self, _id: Uuid, req: StatisticsRequest) -> Result<Value, ServiceError> {
        match req {
            StatisticsRequest::ComputeStatistics(r) => Ok(serde_json::to_value(self.compute(r).await?).expect("serializable")),
            StatisticsRequest::Ping(_) => Ok(Value::Null),
        }
    }
}

impl StatisticsHandler {
    pub async fn compute(&self, r: ComputeStatistics) -> Result<ComputeStatisticsResponse, ServiceError> {
        let sessions = read_all_sessions(&self.bus, &r.filter).await?;
        if sessions.is_empty() {
            return Ok(ComputeStatisticsResponse { groups: vec![], session_count: 0 });
        }
        let used = |f: fn(&Session) -> i64| sessions.iter().map(f).collect::<BTreeSet<_>>().into_iter().collect::<Vec<_>>();
        let devices = if r.group_by == GroupBy::DeviceCategory {
            read_devices(&self.bus, &used(|s| s.device_id)).await?.into_iter().filter_map(|d| Some((d.id?, d))).collect()
        } else {
            HashMap::new()
        };
        let names: HashMap<i64, String> =
            read_target_types(&self.bus, &used(|s| s.target_type_id)).await?.into_iter().filter_map(|t| Some((t.id?, t.name))).collect();
        let session_count = sessions.len() as i64;
        let groups = tokio::task::spawn_blocking(move || compute_groups(&sessions, &r, &devices, &names))
            .await
            .map_err(|e| ServiceError::new(targetscope_common::messages::ErrorCode::ProcessingFailed, e.to_string()))?;
        Ok(ComputeStatisticsResponse { groups, session_count })
    }
}

pub fn compute_groups(
    sessions: &[Session],
    r: &ComputeStatistics,
    devices: &HashMap<i64, targetscope_common::model::Device>,
    names: &HashMap<i64, String>,
) -> Vec<StatisticsGroup> {
    group_sessions(sessions, r.group_by, devices, names)
        .into_iter()
        .map(|g| {
            let summaries: Vec<SessionSummary> = g.sessions.iter().map(|s| summarize(s)).collect();
            let mut periods: BTreeMap<i64, Vec<SessionSummary>> = BTreeMap::new();
            for (s, summary) in g.sessions.iter().zip(&summaries) {
                periods.entry(period_start(s.saved_at, r.period)).or_default().push(*summary);
            }
            StatisticsGroup {
                target_type_id: g.target_type_id,
                group_key: g.group_key,
                summary: aggregate(&summaries),
                trend: periods.into_iter().map(|(period_start, v)| TrendEntry { period_start, values: aggregate(&v) }).collect(),
                shots: distribution_shots(&g.sessions),
            }
        })
        .collect()
}
