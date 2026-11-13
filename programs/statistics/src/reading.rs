// doc/programs/statistics.md#calculation-flow-compute_statistics

use targetscope_common::bus::{Bus, BusError};
use targetscope_common::messages::database_handler::{ReadDevices, ReadSessions, ReadTargetTypes, SessionOrder, SessionOrderField};
use targetscope_common::model::{Device, Direction, Filter, FilterOp, Session, SessionFilter, TargetType};

pub const PAGE_SIZE: i64 = 500;

pub async fn read_all_sessions(bus: &Bus, filter: &SessionFilter) -> Result<Vec<Session>, BusError> {
    let mut all = Vec::new();
    loop {
        let page = bus
            .request(ReadSessions {
                filter: filter.clone(),
                order_by: Some(SessionOrder { field: SessionOrderField::Id, direction: Direction::Asc }),
                limit: Some(PAGE_SIZE),
                offset: Some(all.len() as i64),
                include_shots: true,
            })
            .await?;
        let n = page.sessions.len();
        all.extend(page.sessions);
        if n < PAGE_SIZE as usize || all.len() as i64 >= page.total {
            return Ok(all);
        }
    }
}

fn ids_filter(ids: &[i64]) -> Filter {
    Filter::with("id", FilterOp::In, serde_json::json!(ids))
}

pub async fn read_devices(bus: &Bus, ids: &[i64]) -> Result<Vec<Device>, BusError> {
    Ok(bus.request(ReadDevices { filter: ids_filter(ids) }).await?.devices)
}

pub async fn read_target_types(bus: &Bus, ids: &[i64]) -> Result<Vec<TargetType>, BusError> {
    Ok(bus.request(ReadTargetTypes { filter: ids_filter(ids) }).await?.target_types)
}
