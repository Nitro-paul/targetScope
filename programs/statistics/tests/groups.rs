// doc/testing/statistics.md#groups-integration

mod common;

use std::collections::BTreeSet;

use common::*;
use serde_json::{json, Value};

// ST-GRP-01
#[tokio::test]
async fn st_grp_01_no_grouping() {
    let s = Stats::start(vec![S { id: 1, ..S::default() }.json(), S { id: 2, shooter_id: Some(2), ..S::default() }.json()]).await;
    let groups = s.groups("none").await;
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0]["group_key"], Value::Null);
    assert_eq!(groups[0]["summary"]["session_count"], 2);
}

// ST-GRP-02
#[tokio::test]
async fn st_grp_02_always_split_by_target_type() {
    let s = Stats::start(vec![
        S { id: 1, target_type_id: TT_REF, ..S::default() }.json(),
        S { id: 2, target_type_id: TT_2, ..S::default() }.json(),
    ])
    .await;
    let groups = s.groups("none").await;
    let order: Vec<&Value> = groups.iter().map(|g| &g["target_type_id"]).collect();
    assert_eq!(order, [&json!(TT_2), &json!(TT_REF)]);
}

// ST-GRP-03
#[tokio::test]
async fn st_grp_03_by_shooter() {
    let s = Stats::start(vec![
        S { id: 1, shooter_id: None, ..S::default() }.json(),
        S { id: 2, shooter_id: Some(2), ..S::default() }.json(),
        S { id: 3, shooter_id: Some(1), ..S::default() }.json(),
    ])
    .await;
    assert_eq!(
        keys(&s.groups("shooter").await),
        [json!({ "shooter_id": 1 }), json!({ "shooter_id": 2 }), json!({ "shooter_id": null })]
    );
}

// ST-GRP-04
#[tokio::test]
async fn st_grp_04_by_device() {
    let s = Stats::start(vec![
        S { id: 1, device_id: 3, ..S::default() }.json(),
        S { id: 2, device_id: 1, ..S::default() }.json(),
        S { id: 3, device_id: 3, ..S::default() }.json(),
    ])
    .await;
    let groups = s.groups("device").await;
    assert_eq!(keys(&groups), [json!({ "device_id": 1 }), json!({ "device_id": 3 })]);
    assert_eq!(groups[1]["summary"]["session_count"], 2);
}

// ST-GRP-05
#[tokio::test]
async fn st_grp_05_by_device_category() {
    let s = Stats::start(vec![
        S { id: 1, device_id: 1, ..S::default() }.json(),
        S { id: 2, device_id: 3, ..S::default() }.json(),
        S { id: 3, device_id: 1, ..S::default() }.json(),
    ])
    .await;
    let groups = s.groups("device_category").await;
    assert_eq!(keys(&groups), [json!({ "category": "air pistol" }), json!({ "category": "pistol" })]);
    let asked = s.db.requests_of("read_devices");
    assert_eq!(asked.len(), 1);
    let filter = &asked[0]["filter"]["conditions"][0];
    assert_eq!((filter["field"].as_str(), filter["op"].as_str()), (Some("id"), Some("in")));
    let ids: BTreeSet<i64> = filter["value"].as_array().unwrap().iter().map(|v| v.as_i64().unwrap()).collect();
    assert_eq!(ids, BTreeSet::from([1, 3]));
}

// ST-GRP-06
#[tokio::test]
async fn st_grp_06_by_distance() {
    let s = Stats::start(vec![
        S { id: 1, distance_m: Some(25), ..S::default() }.json(),
        S { id: 2, distance_m: None, ..S::default() }.json(),
        S { id: 3, distance_m: Some(10), ..S::default() }.json(),
    ])
    .await;
    assert_eq!(
        keys(&s.groups("distance").await),
        [json!({ "distance_m": 10 }), json!({ "distance_m": 25 }), json!({ "distance_m": null })]
    );
}

// ST-GRP-07
#[tokio::test]
async fn st_grp_07_by_single_handed_and_supported() {
    let s = Stats::start(vec![
        S { id: 1, single_handed: true, supported: false, ..S::default() }.json(),
        S { id: 2, single_handed: false, supported: true, ..S::default() }.json(),
    ])
    .await;
    assert_eq!(keys(&s.groups("single_handed").await), [json!({ "single_handed": false }), json!({ "single_handed": true })]);
    assert_eq!(keys(&s.groups("supported").await), [json!({ "supported": false }), json!({ "supported": true })]);
}

// ST-GRP-08
#[tokio::test]
async fn st_grp_08_shots_of_a_group() {
    let sessions: Vec<Value> = (1..=100)
        .map(|i| {
            let radius = i as f64 + 0.5;
            S { id: i, saved_at: 1_767_000_000 + i * 60, shots: vec![(0.25, radius); 60], ..S::default() }.json()
        })
        .collect();
    let s = Stats::start(sessions).await;
    let groups = s.groups("none").await;
    let shots = groups[0]["shots"].as_array().unwrap();
    assert_eq!(shots.len(), 5000);
    let sessions_used: BTreeSet<i64> = shots.iter().map(|x| x["radius_mm"].as_f64().unwrap() as i64).collect();
    assert_eq!(sessions_used, (17..=100).collect::<BTreeSet<i64>>());
    let from_17 = shots.iter().filter(|x| x["radius_mm"].as_f64().unwrap() as i64 == 17).count();
    assert_eq!(from_17, 5000 - 83 * 60);
}
