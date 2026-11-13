// doc/testing/shared_libraries.md#settings-cache-integration

use std::time::{Duration, Instant};

use serde_json::{json, Value};
use targetscope_common::bus::Bus;
use targetscope_common::messages::database_handler::{ChangeAction, DataChanged, DatabaseHandlerEvent, Table};
use targetscope_common::settings::SettingsCache;
use targetscope_testkit::{Answer, FakeProgram, RedisTestServer};

fn setting(name: &str, value: &str, active: bool) -> Value {
    json!({ "name": name, "display_name": name, "active": active, "value": value, "optional": !active,
            "default_value": value, "default_active": active, "value_rule": "text" })
}

fn settings(fps: &str, live_fps: &str) -> Answer {
    Answer::Ok(json!({ "settings": [
        setting("camera_fps", fps, true),
        setting("live_feed_fps", live_fps, true),
        setting("export_dir", "~/x", false),
        setting("image_dir", "/srv/images", true),
    ] }))
}

async fn changed(fake: &FakeProgram, table: Table, key: Value, action: ChangeAction) {
    fake.bus.publish(&DatabaseHandlerEvent::DataChanged(DataChanged { table, key, action })).await.unwrap();
}

async fn setup() -> (RedisTestServer, FakeProgram, SettingsCache) {
    let redis = RedisTestServer::start().await;
    let fake = FakeProgram::start(&redis.url(), "database_handler").await;
    fake.on("read_settings", settings("30", "15"));
    let bus = Bus::connect(&redis.url(), "prog").await.unwrap();
    let cache = SettingsCache::load(&bus).await.unwrap();
    (redis, fake, cache)
}

async fn eventually(f: impl Fn() -> bool) -> bool {
    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline {
        if f() {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    false
}

// LIB-SET-01
#[tokio::test(flavor = "multi_thread")]
async fn lib_set_01_load() {
    let (_r, _f, c) = setup().await;
    assert_eq!(c.get("camera_fps").as_deref(), Some("30"));
    assert_eq!(c.get("image_dir").as_deref(), Some("/srv/images"));
}

// LIB-SET-02
#[tokio::test(flavor = "multi_thread")]
async fn lib_set_02_unset_optional_setting() {
    let (_r, _f, c) = setup().await;
    assert_eq!(c.get("export_dir"), None);
}

// LIB-SET-03
#[tokio::test(flavor = "multi_thread")]
async fn lib_set_03_parsed_values() {
    let (_r, f, c) = setup().await;
    assert_eq!(c.get_parsed::<u32>("camera_fps"), Some(30));
    f.on("read_settings", settings("abc", "15"));
    changed(&f, Table::Settings, json!("camera_fps"), ChangeAction::Updated).await;
    assert!(eventually(|| c.get_parsed::<u32>("camera_fps").is_none()).await);
}

// LIB-SET-04
#[tokio::test(flavor = "multi_thread")]
async fn lib_set_04_paths() {
    let c = SettingsCache::from_settings(vec![
        serde_json::from_value(setting("a", "~/TargetScope/exports", true)).unwrap(),
        serde_json::from_value(setting("b", "/srv/exports", true)).unwrap(),
    ]);
    let home = std::path::PathBuf::from(std::env::var("HOME").unwrap());
    assert_eq!(c.get_path("a"), Some(home.join("TargetScope/exports")));
    assert_eq!(c.get_path("b"), Some("/srv/exports".into()));
}

// LIB-SET-05
#[tokio::test(flavor = "multi_thread")]
async fn lib_set_05_change_event() {
    let (_r, f, c) = setup().await;
    f.on("read_settings", settings("15", "15"));
    changed(&f, Table::Settings, json!("camera_fps"), ChangeAction::Updated).await;
    assert!(eventually(|| c.get("camera_fps").as_deref() == Some("15")).await);
}

// LIB-SET-07
#[tokio::test(flavor = "multi_thread")]
async fn lib_set_07_watch() {
    let (_r, f, c) = setup().await;
    let mut rx = c.watch(&["camera_fps"]);
    f.on("read_settings", settings("15", "15"));
    changed(&f, Table::Settings, json!("camera_fps"), ChangeAction::Updated).await;
    tokio::time::timeout(Duration::from_secs(2), rx.changed()).await.unwrap().unwrap();
    f.on("read_settings", settings("15", "5"));
    changed(&f, Table::Settings, json!("live_feed_fps"), ChangeAction::Updated).await;
    assert!(eventually(|| c.get("live_feed_fps").as_deref() == Some("5")).await);
    assert!(!rx.has_changed().unwrap());
}

// LIB-SET-08
#[tokio::test(flavor = "multi_thread")]
async fn lib_set_08_reload_everything() {
    let (_r, f, c) = setup().await;
    f.on("read_settings", settings("10", "10"));
    changed(&f, Table::All, json!(null), ChangeAction::Reload).await;
    assert!(eventually(|| c.get("camera_fps").as_deref() == Some("10") && c.get("live_feed_fps").as_deref() == Some("10")).await);
}
