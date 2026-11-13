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

// doc/testing/database_handler.md#session-filter-integration
pub async fn load_filter_set(bus: &crate::TestBus) -> std::collections::HashMap<String, i64> {
    const DB: &str = "database_handler";
    let set: Value = serde_json::from_str(&std::fs::read_to_string(testdata("sessions/filter_set.json")).unwrap()).unwrap();
    let mut ids = std::collections::HashMap::new();
    let id = |r: targetscope_common::messages::ResponseEnvelope| {
        r.payload.as_ref().and_then(|p| p["id"].as_i64()).unwrap_or_else(|| panic!("{:?}", r.error))
    };
    let records = |kind: &str| set[kind].as_array().unwrap().clone();
    let target_type = |name: &Value| {
        let mut t = serde_json::to_value(tt_ref()).unwrap();
        t["name"] = name.clone();
        t
    };
    for s in records("shooters") {
        ids.insert(s["name"].as_str().unwrap().to_string(), id(bus.send(DB, "save_shooter", json!({ "name": s["name"] })).await));
    }
    for d in records("devices") {
        let mut p = d.clone();
        p.as_object_mut().unwrap().remove("retired");
        ids.insert(d["name"].as_str().unwrap().to_string(), id(bus.send(DB, "save_device", p).await));
    }
    for t in records("target_types") {
        ids.insert(t["name"].as_str().unwrap().to_string(), id(bus.send(DB, "save_target_type", target_type(&t["name"])).await));
    }
    for s in records("sessions") {
        let n = s["shot_count"].as_i64().unwrap();
        let shots: Vec<Value> = (0..n).map(|i| json!({ "angle_turns": i as f64 / n as f64, "radius_mm": 1.5, "paper_no": 1 })).collect();
        let named = |field: &str| s[field].as_str().map(|name| ids[name]);
        let payload = json!({
            "uuid": s["uuid"], "saved_at": s["saved_at"], "score": s["score"], "grouping_score": s["score"],
            "distance_m": s["distance_m"], "target_type_id": named("target_type"), "shooter_id": named("shooter"),
            "device_id": named("device"), "single_handed": s["single_handed"], "supported": s["supported"],
            "shot_count": n, "duration_s": null, "notes": null, "paper_count": 1, "shots": shots, "images": []
        });
        ids.insert(s["name"].as_str().unwrap().to_string(), id(bus.send(DB, "import_session", payload).await));
    }
    for (kind, request) in [("shooters", "save_shooter"), ("devices", "save_device"), ("target_types", "save_target_type")] {
        for r in records(kind).into_iter().filter(|r| r["retired"] == true) {
            let mut p = if kind == "target_types" { target_type(&r["name"]) } else { r.clone() };
            p["id"] = json!(ids[r["name"].as_str().unwrap()]);
            p["retired"] = json!(true);
            id(bus.send(DB, request, p).await);
        }
    }
    ids
}
