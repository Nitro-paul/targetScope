//! Steps 4-7: import_shooter, import_target_type, import_device, import_session in this order,
//! with maps uuid -> local id. Failpoint importer.after_record after each record.

use std::collections::HashMap;

use targetscope_common::bus::Bus;
use targetscope_common::messages::ServiceError;
use uuid::Uuid;

use crate::export_file::ExportFile;

#[derive(Debug, Default, serde::Serialize)]
pub struct Counts {
    pub created: u64,
    pub existing: u64,
}

#[derive(Debug, Default, serde::Serialize)]
pub struct ImportSummary {
    pub shooters: Counts,
    pub target_types: Counts,
    pub devices: Counts,
    pub sessions: Counts,
    pub images: ImageCounts,
    pub warnings: Vec<String>,
}

#[derive(Debug, Default, serde::Serialize)]
pub struct ImageCounts {
    pub restored: u64,
}

pub struct IdMaps {
    pub shooters: HashMap<Uuid, i64>,
    pub target_types: HashMap<Uuid, i64>,
    pub devices: HashMap<Uuid, i64>,
}

pub async fn import_all(bus: &Bus, file: &ExportFile) -> Result<ImportSummary, ServiceError> {
    let _ = (bus, file);
    todo!()
}
