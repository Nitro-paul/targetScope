// doc/testing/shared_libraries.md#settings-cache-integration

use std::time::Duration;

use serde_json::json;
use targetscope_common::bus::Bus;
use targetscope_common::messages::database_handler::{ChangeAction, DataChanged, DatabaseHandlerEvent, Table};
use targetscope_common::settings::SettingsCache;
use targetscope_testkit::failpoints::{self, Action};
use targetscope_testkit::{Answer, FakeProgram, RedisTestServer};

// LIB-SET-06
#[tokio::test(flavor = "multi_thread")]
async fn lib_set_06_no_gap_at_startup() {
    let redis = RedisTestServer::start().await;
    let fake = FakeProgram::start(&redis.url(), "database_handler").await;
    let s = |v: &str| json!({ "settings": [{ "name": "camera_fps", "display_name": "fps", "active": true, "value": v }] });
    fake.on("read_settings", Answer::Ok(s("30")));
    failpoints::set("settings.after_note_id", Action::Sleep(Duration::from_millis(500)));
    let bus = Bus::connect(&redis.url(), "prog").await.unwrap();
    let loading = tokio::spawn(async move { SettingsCache::load(&bus).await.unwrap() });
    tokio::time::sleep(Duration::from_millis(200)).await;
    fake.on("read_settings", Answer::Ok(s("12")));
    let ev =
        DatabaseHandlerEvent::DataChanged(DataChanged { table: Table::Settings, key: json!("camera_fps"), action: ChangeAction::Updated });
    fake.bus.publish(&ev).await.unwrap();
    let cache = loading.await.unwrap();
    failpoints::clear("settings.after_note_id");
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert_eq!(cache.get("camera_fps").as_deref(), Some("12"));
}
