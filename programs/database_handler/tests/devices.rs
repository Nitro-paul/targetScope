// doc/testing/database_handler.md#devices-and-device-images-integration

mod common;

use common::*;
use serde_json::{json, Value};
use targetscope_testkit::fixtures;

fn dev_ref() -> Value {
    serde_json::to_value(fixtures::dev_ref()).unwrap()
}

fn image_name_ok(path: &str, id: i64, ext: &str) -> bool {
    let Some(rest) = path.strip_prefix(&format!("devices/{id}_")) else { return false };
    let Some(hex) = rest.strip_suffix(&format!(".{ext}")) else { return false };
    hex.len() == 8 && hex.chars().all(|c| c.is_ascii_hexdigit())
}

// DB-DEV-01
#[tokio::test]
async fn db_dev_01_create() {
    let h = Handler::start().await;
    let id = h.ok("save_device", dev_ref()).await["id"].as_i64().unwrap();
    let d = h.device(id).await;
    assert!(is_uuid_v4(d["uuid"].as_str().unwrap()));
    assert_eq!(d["shot_count"], 0);
    assert_eq!(d["image_path"], Value::Null);
    assert_eq!(d["locked"], false);
    assert_eq!(d["name"], "Test pistol");
}

// DB-DEV-02
#[tokio::test]
async fn db_dev_02_invalid() {
    let h = Handler::start().await;
    for payload in
        [with(dev_ref(), "name", json!("")), with(dev_ref(), "bullet_diameter_mm", json!(0)), without(dev_ref(), "bullet_diameter_mm")]
    {
        assert_eq!(h.err("save_device", payload).await, "validation_failed");
    }
    assert!(h.ok("read_devices", json!({})).await["devices"].as_array().unwrap().is_empty());
}

// DB-DEV-03
#[tokio::test]
async fn db_dev_03_update_an_unlocked_device() {
    let h = Handler::start().await;
    let id = h.ok("save_device", dev_ref()).await["id"].as_i64().unwrap();
    let changed = json!({
        "id": id, "name": "Other", "category": "air rifle", "description": "new sights", "caliber": ".22",
        "retired": true, "bullet_diameter_mm": 5.5
    });
    h.ok("save_device", changed.clone()).await;
    let d = h.device(id).await;
    for field in ["name", "category", "description", "caliber", "retired", "bullet_diameter_mm"] {
        assert_eq!(d[field], changed[field], "{field}");
    }
}

// DB-DEV-04
#[tokio::test]
async fn db_dev_04_read_only_fields_are_ignored() {
    let h = Handler::start().await;
    let id = h.ok("save_device", dev_ref()).await["id"].as_i64().unwrap();
    let before = h.device(id).await;
    let mut p = dev_ref();
    p["id"] = json!(id);
    p["shot_count"] = json!(999);
    p["image_path"] = json!("x");
    p["locked"] = json!(true);
    p["uuid"] = json!("0b3f1c2e-6c1a-4a8e-9d55-2f6b1d6c7a10");
    h.ok("save_device", p).await;
    let after = h.device(id).await;
    for field in ["shot_count", "image_path", "locked", "uuid"] {
        assert_eq!(after[field], before[field], "{field}");
    }
}

// DB-DEV-05
#[tokio::test]
async fn db_dev_05_locked_device_bullet_diameter() {
    let h = Handler::start().await;
    let ids = h.refs().await;
    h.create_session(ids).await;
    let before = h.device(ids.device_id).await;
    let p = with(with(dev_ref(), "id", json!(ids.device_id)), "bullet_diameter_mm", json!(5.5));
    assert_eq!(h.err("save_device", p).await, "in_use");
    assert_eq!(h.device(ids.device_id).await, before);
}

// DB-DEV-06
#[tokio::test]
async fn db_dev_06_locked_device_other_fields() {
    let h = Handler::start().await;
    let ids = h.refs().await;
    h.create_session(ids).await;
    let p = with(with(with(dev_ref(), "id", json!(ids.device_id)), "name", json!("Renamed")), "description", json!("d"));
    h.ok("save_device", p).await;
    let d = h.device(ids.device_id).await;
    assert_eq!(d["name"], "Renamed");
    assert_eq!(d["description"], "d");
}

// DB-DEV-07
#[tokio::test]
async fn db_dev_07_locked_flag() {
    let h = Handler::start().await;
    let ids = h.refs().await;
    assert_eq!(h.device(ids.device_id).await["locked"], false);
    h.create_session(ids).await;
    assert_eq!(h.device(ids.device_id).await["locked"], true);
}

// DB-DEV-08
#[tokio::test]
async fn db_dev_08_unknown_id() {
    let h = Handler::start().await;
    assert_eq!(h.err("save_device", with(dev_ref(), "id", json!(999))).await, "not_found");
}

// DB-DEV-09
#[tokio::test]
async fn db_dev_09_set_an_image() {
    let h = Handler::start().await;
    let id = h.ok("save_device", dev_ref()).await["id"].as_i64().unwrap();
    let staged = h.stage("paper.jpg");
    let mut probe = h.probe().await;
    let path =
        h.ok("set_device_image", json!({ "device_id": id, "staged_file": staged })).await["image_path"].as_str().unwrap().to_string();
    assert!(image_name_ok(&path, id, "jpg"), "{path}");
    assert_eq!(h.device(id).await["image_path"], path.as_str());
    assert_eq!(std::fs::read(h.image(&path)).unwrap(), std::fs::read(fixtures::testdata("images/paper.jpg")).unwrap());
    assert!(!h.dir.staging_dir.join(&staged).exists());
    assert_eq!(changes(&mut probe, 1).await, [format!("devices/{id}/updated")]);
}

// DB-DEV-10
#[tokio::test]
async fn db_dev_10_replace_an_image_twice_quickly() {
    let h = Handler::start().await;
    let id = h.ok("save_device", dev_ref()).await["id"].as_i64().unwrap();
    let a = h.stage("paper.jpg");
    let b = h.stage("paper.jpg");
    let first = h.ok("set_device_image", json!({ "device_id": id, "staged_file": a })).await["image_path"].clone();
    let second = h.ok("set_device_image", json!({ "device_id": id, "staged_file": b })).await["image_path"].clone();
    assert_ne!(first, second);
    assert_eq!(h.device(id).await["image_path"], second);
    assert!(!h.image(first.as_str().unwrap()).exists());
    assert!(h.image(second.as_str().unwrap()).exists());
}

// DB-DEV-11
#[tokio::test]
async fn db_dev_11_image_for_an_unknown_device() {
    let h = Handler::start().await;
    let staged = h.stage("paper.jpg");
    assert_eq!(h.err("set_device_image", json!({ "device_id": 999, "staged_file": staged })).await, "not_found");
    assert!(h.dir.staging_dir.join(&staged).exists());
}

// DB-DEV-12
#[tokio::test]
async fn db_dev_12_remove_the_image() {
    let h = Handler::start().await;
    let id = h.ok("save_device", dev_ref()).await["id"].as_i64().unwrap();
    let staged = h.stage("paper.jpg");
    let path = h.ok("set_device_image", json!({ "device_id": id, "staged_file": staged })).await["image_path"].clone();
    assert_eq!(h.ok("delete_device_image", json!({ "device_id": id })).await["device_id"], id);
    assert_eq!(h.device(id).await["image_path"], Value::Null);
    assert!(!h.image(path.as_str().unwrap()).exists());
}

// DB-DEV-13
#[tokio::test]
async fn db_dev_13_remove_when_there_is_none() {
    let h = Handler::start().await;
    let id = h.ok("save_device", dev_ref()).await["id"].as_i64().unwrap();
    h.ok("delete_device_image", json!({ "device_id": id })).await;
    assert_eq!(h.device(id).await["image_path"], Value::Null);
}

// DB-DEV-14
#[tokio::test]
async fn db_dev_14_delete_unused_device() {
    let h = Handler::start().await;
    let id = h.ok("save_device", dev_ref()).await["id"].as_i64().unwrap();
    let staged = h.stage("device.png");
    let path = h.ok("set_device_image", json!({ "device_id": id, "staged_file": staged })).await["image_path"].clone();
    assert!(image_name_ok(path.as_str().unwrap(), id, "png"));
    let mut probe = h.probe().await;
    assert_eq!(h.ok("delete_device", json!({ "id": id })).await["id"], id);
    assert_eq!(h.count("SELECT count(*) FROM devices"), 0);
    assert!(!h.image(path.as_str().unwrap()).exists());
    assert_eq!(changes(&mut probe, 1).await, [format!("devices/{id}/deleted")]);
}

// DB-DEV-15
#[tokio::test]
async fn db_dev_15_delete_used_device() {
    let h = Handler::start().await;
    let ids = h.refs().await;
    h.create_session(ids).await;
    assert_eq!(h.err("delete_device", json!({ "id": ids.device_id })).await, "in_use");
}
