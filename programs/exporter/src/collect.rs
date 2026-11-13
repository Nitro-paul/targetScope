// doc/programs/exporter.md#export-flow-export_sessions

use std::collections::{BTreeSet, HashMap};

use targetscope_common::bus::Bus;
use targetscope_common::messages::analyzer::AnalyzeShots;
use targetscope_common::messages::database_handler::{
    ReadDevices, ReadSessions, ReadShooters, ReadTargetTypes, SessionOrder, SessionOrderField,
};
use targetscope_common::messages::ServiceError;
use targetscope_common::model::{AnalysisResult, Device, Direction, Filter, FilterOp, Session, SessionFilter, Shooter, TargetType};

pub const PAGE_SIZE: i64 = 500;

pub struct ExportData {
    pub sessions: Vec<(Session, AnalysisResult)>,
    pub shooters: HashMap<i64, Shooter>,
    pub devices: HashMap<i64, Device>,
    pub target_types: HashMap<i64, TargetType>,
}

fn ids(values: impl Iterator<Item = i64>) -> Filter {
    let list: Vec<i64> = values.collect::<BTreeSet<_>>().into_iter().collect();
    Filter::with("id", FilterOp::In, serde_json::json!(list))
}

pub async fn collect(bus: &Bus, filter: &SessionFilter) -> Result<ExportData, ServiceError> {
    let mut sessions = Vec::new();
    loop {
        let page = bus
            .request(ReadSessions {
                filter: filter.clone(),
                order_by: Some(SessionOrder { field: SessionOrderField::Id, direction: Direction::Asc }),
                limit: Some(PAGE_SIZE),
                offset: Some(sessions.len() as i64),
                include_shots: true,
            })
            .await?;
        let n = page.sessions.len();
        sessions.extend(page.sessions);
        if n < PAGE_SIZE as usize || sessions.len() as i64 >= page.total {
            break;
        }
    }
    if sessions.is_empty() {
        return Err(ServiceError::not_found("no session matches the filter"));
    }
    let shooters = bus.request(ReadShooters { filter: ids(sessions.iter().filter_map(|s| s.shooter_id)) }).await?.shooters;
    let devices = bus.request(ReadDevices { filter: ids(sessions.iter().map(|s| s.device_id)) }).await?.devices;
    let target_types = bus.request(ReadTargetTypes { filter: ids(sessions.iter().map(|s| s.target_type_id)) }).await?.target_types;
    let shooters: HashMap<i64, Shooter> = shooters.into_iter().filter_map(|s| Some((s.id?, s))).collect();
    let devices: HashMap<i64, Device> = devices.into_iter().filter_map(|d| Some((d.id?, d))).collect();
    let target_types: HashMap<i64, TargetType> = target_types.into_iter().filter_map(|t| Some((t.id?, t))).collect();
    let mut analyzed = Vec::with_capacity(sessions.len());
    for s in sessions {
        let (Some(t), Some(d)) = (target_types.get(&s.target_type_id), devices.get(&s.device_id)) else {
            return Err(ServiceError::not_found(format!("records of session {}", s.id)));
        };
        let req = AnalyzeShots {
            shots: s.shots.iter().map(|x| x.shot()).collect(),
            target_type: t.clone(),
            bullet_diameter_mm: d.bullet_diameter_mm,
        };
        let result = bus.request(req).await?;
        analyzed.push((s, result));
    }
    Ok(ExportData { sessions: analyzed, shooters, devices, target_types })
}
