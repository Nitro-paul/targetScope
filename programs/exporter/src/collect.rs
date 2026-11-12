//! Steps 1-3 of export_sessions: read_sessions in pages of 500 ordered by id; read shooters,
//! devices and target types used by them; analyze_shots per session for shot scores and shift.

use targetscope_common::bus::Bus;
use targetscope_common::messages::ServiceError;
use targetscope_common::model::{
    AnalysisResult, Device, Session, SessionFilter, Shooter, TargetType,
};

pub struct ExportData {
    pub sessions: Vec<(Session, AnalysisResult)>,
    pub shooters: Vec<Shooter>,
    pub devices: Vec<Device>,
    pub target_types: Vec<TargetType>,
}

/// not_found if no session matches.
pub async fn collect(bus: &Bus, filter: &SessionFilter) -> Result<ExportData, ServiceError> {
    let _ = (bus, filter);
    todo!()
}
