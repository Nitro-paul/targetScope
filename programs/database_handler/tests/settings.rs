// doc/testing/database_handler.md#settings-integration

mod common;

use std::os::unix::fs::PermissionsExt;

use common::*;
use serde_json::{json, Value};

const PHC: &str = "$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$aGFzaGhhc2hoYXNoaGFzaGhhc2hoYXNoaGFzaGhhc2g";

async fn save(h: &Handler, name: &str, value: &str, active: bool) -> Value {
    h.send("save_setting", setting(name, value, active)).await.payload.unwrap_or(Value::Null)
}

fn setting(name: &str, value: &str, active: bool) -> Value {
    json!({ "name": name, "display_name": name, "value": value, "active": active })
}

async fn save_ok(h: &Handler, name: &str, value: &str) {
    h.ok("save_setting", setting(name, value, true)).await;
}

async fn all_settings(h: &Handler) -> Vec<Value> {
    h.ok("read_settings", json!({})).await["settings"].as_array().unwrap().clone()
}

// DB-SET-01
#[tokio::test]
async fn db_set_01_fields_of_a_setting() {
    let h = Handler::start().await;
    let settings = all_settings(&h).await;
    assert_eq!(settings.len(), 20);
    for s in settings {
        for field in ["optional", "default_value", "default_active", "value_rule"] {
            assert!(s.get(field).is_some(), "{}: {field}", s["name"]);
        }
    }
}

// DB-SET-02
#[tokio::test]
async fn db_set_02_valid_values() {
    let h = Handler::start().await;
    let export_dir = h.dir.export_dir.display().to_string();
    let cases = [
        ("export_dir", export_dir.as_str()),
        ("camera_fps", "1"),
        ("camera_fps", "120"),
        ("camera_source", "rtsp"),
        ("camera_rtsp_url", "rtsp://192.168.1.5/stream"),
        ("web_bind_address", "192.168.1.10:80"),
        ("camera_controls", "{}"),
        ("camera_device", "usb-0000:00:14.0-2"),
    ];
    let mut probe = h.probe().await;
    for (name, value) in cases {
        save_ok(&h, name, value).await;
        assert_eq!(h.setting(name).await["value"], value);
        assert_eq!(changes(&mut probe, 1).await, [format!("settings/{name}/updated")]);
    }
}

// DB-SET-03
#[tokio::test]
async fn db_set_03_invalid_values() {
    let h = Handler::start().await;
    let cases = [
        ("camera_fps", "0"),
        ("camera_fps", "121"),
        ("camera_fps", "abc"),
        ("camera_fps", "30.5"),
        ("camera_source", "usb"),
        ("camera_rtsp_url", "http://x"),
        ("web_bind_address", "8080"),
        ("web_bind_address", "0.0.0.0:70000"),
        ("web_bind_address", "localhost:8080"),
        ("camera_controls", "[]"),
        ("camera_controls", "{"),
        ("live_feed_quality", "101"),
        ("admin_password_hash", "secret"),
    ];
    for (name, value) in cases {
        let before = h.setting(name).await;
        assert_eq!(h.err("save_setting", setting(name, value, true)).await, "validation_failed", "{name} = {value}");
        assert_eq!(h.setting(name).await, before);
    }
}

// DB-SET-04
#[tokio::test]
async fn db_set_04_unsetting_a_required_setting() {
    let h = Handler::start().await;
    assert_eq!(h.err("save_setting", setting("camera_fps", "30", false)).await, "validation_failed");
}

// DB-SET-05
#[tokio::test]
async fn db_set_05_unsetting_an_optional_setting() {
    let h = Handler::start().await;
    let dir = h.dir.export_dir.display().to_string();
    save_ok(&h, "export_dir", &dir).await;
    h.ok("save_setting", setting("export_dir", &dir, false)).await;
    let s = h.setting("export_dir").await;
    assert_eq!(s["active"], false);
    assert_eq!(s["value"], dir.as_str());
}

// DB-SET-06
#[tokio::test]
async fn db_set_06_image_dir_through_save_setting() {
    let h = Handler::start().await;
    let r = h.send("save_setting", setting("image_dir", &h.dir.root.join("new").display().to_string(), true)).await;
    let e = r.error.unwrap();
    assert_eq!(e.code.as_str(), "validation_failed");
    assert!(e.message.contains("change_image_dir"), "{}", e.message);
}

// DB-SET-07
#[tokio::test]
async fn db_set_07_staging_folder_on_another_file_system() {
    let h = Handler::start().await;
    let Some(other) = h.dir.other_file_system() else {
        eprintln!("no second file system available; DB-SET-07 not checked");
        return;
    };
    let before = h.setting("staging_dir").await;
    assert_eq!(h.err("save_setting", setting("staging_dir", &other.display().to_string(), true)).await, "validation_failed");
    assert_eq!(h.setting("staging_dir").await, before);
    let _ = std::fs::remove_dir_all(other);
}

// DB-SET-08
#[tokio::test]
async fn db_set_08_folder_can_t_be_written() {
    let h = Handler::start().await;
    let locked = h.dir.root.join("locked");
    std::fs::create_dir(&locked).unwrap();
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o555)).unwrap();
    let r = save(&h, "export_dir", &locked.join("exports").display().to_string(), true).await;
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert_eq!(r, Value::Null);
    assert_ne!(h.setting("export_dir").await["value"], locked.join("exports").display().to_string());
}

// DB-SET-09
#[tokio::test]
async fn db_set_09_folder_doesn_t_exist_yet() {
    let h = Handler::start().await;
    let dir = h.dir.root.join("new/exports");
    save_ok(&h, "export_dir", &dir.display().to_string()).await;
    assert!(dir.is_dir());
}

// DB-SET-10
#[tokio::test]
async fn db_set_10_delete_a_known_setting() {
    let h = Handler::start().await;
    assert_eq!(h.err("delete_setting", json!({ "name": "camera_fps" })).await, "validation_failed");
}

// DB-SET-11
#[tokio::test]
async fn db_set_11_own_setting() {
    let h = Handler::start().await;
    save_ok(&h, "my_setting", "x").await;
    assert_eq!(h.ok("delete_setting", json!({ "name": "my_setting" })).await["name"], "my_setting");
    assert_eq!(h.err("delete_setting", json!({ "name": "my_setting" })).await, "not_found");
}

// DB-SET-12
#[tokio::test]
async fn db_set_12_reset_by_name() {
    let h = Handler::start().await;
    save_ok(&h, "camera_fps", "15").await;
    save_ok(&h, "export_dir", &h.dir.export_dir.display().to_string()).await;
    let mut probe = h.probe().await;
    h.ok("reset_settings", json!({ "names": ["camera_fps", "export_dir"] })).await;
    for name in ["camera_fps", "export_dir"] {
        let s = h.setting(name).await;
        assert_eq!(s["value"], s["default_value"], "{name}");
        assert_eq!(s["active"], s["default_active"], "{name}");
    }
    let mut events = changes(&mut probe, 2).await;
    events.sort();
    assert_eq!(events, ["settings/camera_fps/updated", "settings/export_dir/updated"]);
}

// DB-SET-13
#[tokio::test]
async fn db_set_13_reset_all() {
    let h = Handler::start().await;
    save_ok(&h, "camera_fps", "15").await;
    save_ok(&h, "live_feed_quality", "50").await;
    save_ok(&h, "camera_source", "none").await;
    save_ok(&h, "admin_password_hash", PHC).await;
    save_ok(&h, "lens_calibrations", r#"{"cam":{"rms_px":0.3}}"#).await;
    save_ok(&h, "my_setting", "mine").await;
    h.ok("reset_settings", json!({ "names": null })).await;
    for s in all_settings(&h).await {
        let name = s["name"].as_str().unwrap();
        match name {
            "admin_password_hash" => assert_eq!(s["value"], PHC),
            "lens_calibrations" => assert_eq!(s["value"], r#"{"cam":{"rms_px":0.3}}"#),
            "my_setting" => assert_eq!(s["value"], "mine"),
            _ => {
                assert_eq!(s["value"], s["default_value"], "{name}");
                assert_eq!(s["active"], s["default_active"], "{name}");
            }
        }
    }
}

// DB-SET-14
#[tokio::test]
async fn db_set_14_reset_an_unknown_name() {
    let h = Handler::start().await;
    save_ok(&h, "camera_fps", "15").await;
    assert_eq!(h.err("reset_settings", json!({ "names": ["camera_fps", "nonsense"] })).await, "validation_failed");
    assert_eq!(h.setting("camera_fps").await["value"], "15");
}

// DB-SET-15
#[tokio::test]
async fn db_set_15_reset_the_password_by_name() {
    let h = Handler::start().await;
    save_ok(&h, "admin_password_hash", PHC).await;
    h.ok("reset_settings", json!({ "names": ["admin_password_hash"] })).await;
    let s = h.setting("admin_password_hash").await;
    assert_eq!(s["active"], false);
    assert_eq!(s["value"], "");
}
