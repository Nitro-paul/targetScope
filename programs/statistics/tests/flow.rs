// doc/testing/statistics.md#flow-and-requests-integration

mod common;

use std::time::Duration;

use common::*;
use serde_json::{json, Value};
use targetscope_testkit::{fixtures, Answer, FakeProgram, ProgramUnderTest, RedisTestServer, TempDataDir, TestBus};

// ST-FLW-01
#[tokio::test]
async fn st_flw_01_filter_passed_on() {
    let s = Stats::start(vec![S::default().json()]).await;
    let filter = json!({
        "session_ids": [1, 2], "shooter_ids": [1], "device_ids": [1, 3], "device_categories": ["air pistol"],
        "calibers": [".177"], "target_type_ids": [1], "distance_m": { "min": 5, "max": 25 },
        "saved_at": { "from": 1_700_000_000, "to": 1_800_000_000 }, "single_handed": true, "supported": false,
        "shot_count": { "min": 1, "max": 60 }, "include_retired": true
    });
    s.ok(json!({ "filter": filter, "group_by": "none", "period": "session" })).await;
    let asked = s.db.requests_of("read_sessions");
    assert_eq!(asked.len(), 1);
    assert_eq!(asked[0]["filter"], filter);
    assert_eq!(asked[0]["include_shots"], true);
    assert_eq!(asked[0]["order_by"], json!({ "field": "id", "direction": "asc" }));
}

// ST-FLW-02
#[tokio::test]
async fn st_flw_02_paging() {
    let sessions: Vec<Value> = (1..=1234).map(|i| S { id: i, saved_at: 1_767_000_000 + i, ..S::default() }.json()).collect();
    let s = Stats::start(sessions).await;
    let r = s.ok(json!({ "filter": {}, "group_by": "none", "period": "month" })).await;
    let offsets: Vec<Value> = s.db.requests_of("read_sessions").iter().map(|p| p["offset"].clone()).collect();
    assert_eq!(offsets, [json!(0), json!(500), json!(1000)]);
    assert_eq!(r["session_count"], 1234);
    assert_eq!(r["groups"][0]["summary"]["session_count"], 1234);
}

// ST-FLW-03
#[tokio::test]
async fn st_flw_03_no_sessions() {
    let s = Stats::start(vec![]).await;
    let r = s.ok(json!({ "filter": {}, "group_by": "shooter", "period": "week" })).await;
    assert_eq!(r, json!({ "groups": [], "session_count": 0 }));
}

// ST-FLW-04
#[tokio::test]
async fn st_flw_04_unknown_ids() {
    let s = Stats::start(vec![]).await;
    let r = s.ok(json!({ "filter": { "shooter_ids": [999] }, "group_by": "shooter", "period": "week" })).await;
    assert_eq!(r["groups"], json!([]));
}

// ST-FLW-05
#[tokio::test]
async fn st_flw_05_database_handler_silent() {
    let redis = RedisTestServer::start().await;
    let dir = TempDataDir::new();
    let db = FakeProgram::start(&redis.url(), "database_handler").await;
    db.on("read_sessions", Answer::NoAnswer);
    let _p = start_statistics(&redis, &dir).await;
    let bus = TestBus::connect(&redis.url()).await;
    let r = bus
        .try_send(P, uuid::Uuid::new_v4(), "compute_statistics", json!({ "filter": {}, "group_by": "none", "period": "day" }), Duration::from_secs(30))
        .await
        .expect("response");
    assert_eq!(r.error.unwrap().code.as_str(), "timeout");
}

// ST-FLW-06
#[tokio::test]
#[ignore = "needs testdata/sessions/statistics_set.json and its hand-checked expected result"]
async fn st_flw_06_whole_result() {}

// ST-FLW-07
#[tokio::test]
async fn st_flw_07_retired_records() {
    let redis = RedisTestServer::start().await;
    let db_dir = TempDataDir::new();
    let _db = ProgramUnderTest::start(&targetscope_testkit::process::program_binary("database_handler"), "database_handler", &db_dir.env(&redis.url())).await;
    let bus = TestBus::connect(&redis.url()).await;
    fixtures::load_filter_set(&bus).await;
    let dir = TempDataDir::new();
    let _p = start_statistics(&redis, &dir).await;
    let count = |include_retired: bool| {
        let bus = &bus;
        async move {
            let payload = json!({ "filter": { "include_retired": include_retired }, "group_by": "shooter", "period": "month" });
            let r = bus.try_send(P, uuid::Uuid::new_v4(), "compute_statistics", payload, Duration::from_secs(90)).await.unwrap();
            r.payload.unwrap()["session_count"].as_i64().unwrap()
        }
    };
    assert_eq!(count(false).await, 5);
    assert_eq!(count(true).await, 8);
}
