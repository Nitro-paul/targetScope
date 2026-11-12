// doc/testing/overview.md#reference-data

use std::path::{Path, PathBuf};

use serde_json::{json, Value};
use targetscope_common::model::{Device, Shooter, TargetType};

pub fn testdata(rel: &str) -> PathBuf {
    crate::testdata_dir().join(rel)
}

// doc/interfaces.md#staging-folder
pub fn stage_image(staging_dir: &Path, name: &str) -> String {
    let ext = Path::new(name).extension().and_then(|e| e.to_str()).unwrap_or("jpg");
    let staged = format!("upload_{}.{ext}", uuid::Uuid::new_v4());
    std::fs::copy(testdata(&format!("images/{name}")), staging_dir.join(&staged)).expect("copy fixture image");
    staged
}

pub fn stage_bytes(staging_dir: &Path, name: &str, bytes: &[u8]) -> String {
    std::fs::write(staging_dir.join(name), bytes).unwrap();
    name.to_string()
}

pub fn reference() -> Value {
    serde_json::from_str(&std::fs::read_to_string(testdata("sessions/reference.json")).unwrap()).unwrap()
}

// doc/testing/overview.md#reference-target-type-tt-ref
pub fn tt_ref() -> TargetType {
    let v: Value = serde_json::from_str(&std::fs::read_to_string(testdata("target_types/tt_ref.json")).unwrap()).unwrap();
    serde_json::from_value(v["target_type"].clone()).unwrap()
}

// doc/testing/overview.md#other-reference-records
pub fn sh_ref() -> Shooter {
    serde_json::from_value(reference()["shooter"].clone()).unwrap()
}

pub fn dev_ref() -> Device {
    serde_json::from_value(reference()["device"].clone()).unwrap()
}

pub fn s_ref_shots() -> Vec<Value> {
    reference()["session"]["shots"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| json!({ "angle_turns": s["angle_turns"], "radius_mm": s["radius_mm"], "paper_no": s["paper_no"] }))
        .collect()
}

// doc/testing/overview.md#test-kit
pub async fn create_reference_records(bus: &crate::TestBus) -> ReferenceIds {
    let id = |r: targetscope_common::messages::ResponseEnvelope| {
        r.payload.and_then(|p| p["id"].as_i64()).unwrap_or_else(|| panic!("{:?}", r.error))
    };
    ReferenceIds {
        target_type_id: id(bus.send("database_handler", "save_target_type", serde_json::to_value(tt_ref()).unwrap()).await),
        shooter_id: id(bus.send("database_handler", "save_shooter", serde_json::to_value(sh_ref()).unwrap()).await),
        device_id: id(bus.send("database_handler", "save_device", serde_json::to_value(dev_ref()).unwrap()).await),
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ReferenceIds {
    pub target_type_id: i64,
    pub shooter_id: i64,
    pub device_id: i64,
}
