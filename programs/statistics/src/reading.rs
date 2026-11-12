//! Step 1-2 of compute_statistics: read_sessions in pages of 500 ordered by id
//! (include_shots = true), then read_devices of the sessions.

use targetscope_common::bus::{Bus, BusError};
use targetscope_common::model::{Device, Session, SessionFilter};

pub const PAGE_SIZE: i64 = 500;

pub async fn read_all_sessions(
    bus: &Bus,
    filter: &SessionFilter,
) -> Result<Vec<Session>, BusError> {
    let _ = (bus, filter);
    todo!()
}

pub async fn read_devices(bus: &Bus, ids: &[i64]) -> Result<Vec<Device>, BusError> {
    let _ = (bus, ids);
    todo!()
}
