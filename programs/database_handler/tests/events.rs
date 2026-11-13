// doc/testing/database_handler.md#change-events-integration

mod common;

use common::*;
use serde_json::{json, Value};
use targetscope_testkit::fixtures;

fn sorted(mut v: Vec<String>) -> Vec<String> {
    v.sort();
    v
}

fn dev_ref() -> Value {
    serde_json::to_value(fixtures::dev_ref()).unwrap()
}

fn tt_ref() -> Value {
    serde_json::to_value(fixtures::tt_ref()).unwrap()
}

// DB-EVT-01
#[tokio::test]
async fn db_evt_01_session_created() {
    let h = Handler::start().await;
    let ids = h.refs().await;
    let mut probe = h.probe().await;
    let id = h.create_session(ids).await["id"].as_i64().unwrap();
    assert_eq!(
        sorted(changes(&mut probe, 2).await),
        sorted(vec![format!("sessions/{id}/created"), format!("devices/{}/updated", ids.device_id)])
    );
}

// DB-EVT-02
#[tokio::test]
async fn db_evt_02_session_deleted() {
    let h = Handler::start().await;
    let ids = h.refs().await;
    let id = h.create_session(ids).await["id"].as_i64().unwrap();
    let mut probe = h.probe().await;
    h.ok("delete_session", json!({ "id": id })).await;
    assert_eq!(changes(&mut probe, 1).await, [format!("sessions/{id}/deleted")]);
    probe.expect_none(NO_EVENT_WAIT).await;
}

// DB-EVT-03
#[tokio::test]
async fn db_evt_03_shooters() {
    let h = Handler::start().await;
    let mut probe = h.probe().await;
    let id = h.ok("save_shooter", json!({ "name": "Alex" })).await["id"].as_i64().unwrap();
    h.ok("save_shooter", json!({ "id": id, "name": "Bea" })).await;
    h.ok("delete_shooter", json!({ "id": id })).await;
    let expected: Vec<String> = ["created", "updated", "deleted"].iter().map(|a| format!("shooters/{id}/{a}")).collect();
    assert_eq!(changes(&mut probe, 3).await, expected);
}

// DB-EVT-04
#[tokio::test]
async fn db_evt_04_devices() {
    let h = Handler::start().await;
    let mut probe = h.probe().await;
    let id = h.ok("save_device", dev_ref()).await["id"].as_i64().unwrap();
    h.ok("save_device", with(with(dev_ref(), "id", json!(id)), "name", json!("Other"))).await;
    let staged = h.stage("paper.jpg");
    h.ok("set_device_image", json!({ "device_id": id, "staged_file": staged })).await;
    h.ok("delete_device_image", json!({ "device_id": id })).await;
    h.ok("delete_device", json!({ "id": id })).await;
    let expected: Vec<String> =
        ["created", "updated", "updated", "updated", "deleted"].iter().map(|a| format!("devices/{id}/{a}")).collect();
    assert_eq!(changes(&mut probe, 5).await, expected);
}

// DB-EVT-05
#[tokio::test]
async fn db_evt_05_target_types() {
    let h = Handler::start().await;
    let mut probe = h.probe().await;
    let id = h.ok("save_target_type", tt_ref()).await["id"].as_i64().unwrap();
    h.ok("save_target_type", with(with(tt_ref(), "id", json!(id)), "name", json!("TT-2"))).await;
    h.ok("delete_target_type", json!({ "id": id })).await;
    let expected: Vec<String> = ["created", "updated", "deleted"].iter().map(|a| format!("target_types/{id}/{a}")).collect();
    assert_eq!(changes(&mut probe, 3).await, expected);
}

// DB-EVT-06
#[tokio::test]
async fn db_evt_06_settings() {
    let h = Handler::start().await;
    let mut probe = h.probe().await;
    let setting = |name: &str, value: &str| json!({ "name": name, "display_name": name, "value": value, "active": true });
    h.ok("save_setting", setting("camera_fps", "15")).await;
    h.ok("save_setting", setting("my_setting", "x")).await;
    h.ok("delete_setting", json!({ "name": "my_setting" })).await;
    assert_eq!(changes(&mut probe, 3).await, ["settings/camera_fps/updated", "settings/my_setting/created", "settings/my_setting/deleted"]);
}

// DB-EVT-07
#[tokio::test]
async fn db_evt_07_reset() {
    let h = Handler::start().await;
    let mut probe = h.probe().await;
    h.ok("reset_settings", json!({ "names": ["camera_fps", "camera_source", "live_feed_fps"] })).await;
    assert_eq!(
        sorted(changes(&mut probe, 3).await),
        ["settings/camera_fps/updated", "settings/camera_source/updated", "settings/live_feed_fps/updated"]
    );
}

// DB-EVT-08
#[tokio::test]
async fn db_evt_08_image_folder_change() {
    let h = Handler::start().await;
    let mut probe = h.probe().await;
    let new_dir = h.dir.data_dir.join("images2");
    h.ok("change_image_dir", json!({ "path": new_dir.display().to_string(), "move_images": false })).await;
    assert_eq!(changes(&mut probe, 1).await, ["settings/image_dir/updated"]);
}

// DB-EVT-09
#[tokio::test]
async fn db_evt_09_imports() {
    let h = Handler::start().await;
    let ids = h.refs().await;
    let mut probe = h.probe().await;
    let shooter = json!({ "uuid": "11111111-1111-4111-8111-111111111111", "name": "Imported" });
    let target_type = with(with(tt_ref(), "uuid", json!("22222222-2222-4222-8222-222222222222")), "name", json!("TT-IMP"));
    let device = with(with(dev_ref(), "uuid", json!("33333333-3333-4333-8333-333333333333")), "name", json!("Imported"));
    let mut session = cs_ref(ids, None);
    session["uuid"] = json!("44444444-4444-4444-8444-444444444444");
    session["saved_at"] = json!(1746093600);
    let mut expected = Vec::new();
    for (request, payload, table) in [
        ("import_shooter", shooter, "shooters"),
        ("import_target_type", target_type, "target_types"),
        ("import_device", device, "devices"),
        ("import_session", session, "sessions"),
    ] {
        let first = h.ok(request, payload.clone()).await;
        assert_eq!(first["created"], true);
        expected.push(format!("{table}/{}/created", first["id"]));
        assert_eq!(h.ok(request, payload).await["created"], false);
    }
    assert_eq!(changes(&mut probe, 4).await, expected);
    probe.expect_none(NO_EVENT_WAIT).await;
}

// DB-EVT-10
#[tokio::test]
async fn db_evt_10_no_event_without_a_write() {
    let h = Handler::start().await;
    let ids = h.refs().await;
    let session = h.create_session(ids).await["id"].as_i64().unwrap();
    let mut probe = h.probe().await;
    for (request, payload) in [
        ("read_sessions", json!({ "filter": {} })),
        ("read_shooters", json!({})),
        ("read_devices", json!({})),
        ("read_target_types", json!({})),
        ("read_settings", json!({})),
    ] {
        h.ok(request, payload).await;
    }
    for (request, payload) in [
        ("save_shooter", json!({ "name": "" })),
        ("save_device", json!({ "id": 999, "name": "x", "bullet_diameter_mm": 4.5 })),
        ("save_target_type", with(tt_ref(), "outer_score", json!(0))),
        ("delete_session", json!({ "id": session + 1 })),
        ("delete_shooter", json!({ "id": ids.shooter_id })),
        ("save_setting", json!({ "name": "camera_fps", "display_name": "x", "value": "0", "active": true })),
        ("reset_settings", json!({ "names": ["nonsense"] })),
        ("create_session", cs_ref(ids, Some("upload_missing.jpg"))),
    ] {
        h.err(request, payload).await;
    }
    probe.expect_none(NO_EVENT_WAIT).await;
}
