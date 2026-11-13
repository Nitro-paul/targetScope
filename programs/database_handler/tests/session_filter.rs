// doc/testing/database_handler.md#session-filter-integration

mod common;

use std::collections::HashMap;

use common::*;
use serde_json::{json, Value};
use targetscope_testkit::fixtures;

struct FilterSet {
    h: Handler,
    ids: HashMap<String, i64>,
}

impl FilterSet {
    async fn load() -> Self {
        let h = Handler::start().await;
        let ids = fixtures::load_filter_set(&h.bus).await;
        FilterSet { h, ids }
    }

    fn id(&self, name: &str) -> i64 {
        self.ids[name]
    }

    fn name_of(&self, id: i64) -> String {
        self.ids.iter().find(|(k, v)| **v == id && k.starts_with('F')).map(|(k, _)| k.clone()).unwrap()
    }

    async fn read(&self, payload: Value) -> (Vec<String>, i64) {
        let r = self.h.ok("read_sessions", payload).await;
        let names = r["sessions"].as_array().unwrap().iter().map(|s| self.name_of(s["id"].as_i64().unwrap())).collect();
        (names, r["total"].as_i64().unwrap())
    }

    async fn matching(&self, filter: Value) -> Vec<String> {
        let (mut names, total) = self.read(json!({ "filter": filter, "include_shots": false })).await;
        assert_eq!(total, names.len() as i64);
        names.sort();
        names
    }
}

fn names(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

// DB-FLT-01
#[tokio::test]
async fn db_flt_01_no_restriction() {
    let f = FilterSet::load().await;
    assert_eq!(f.matching(json!({})).await, names(&["F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8"]));
}

// DB-FLT-02
#[tokio::test]
async fn db_flt_02_shooters() {
    let f = FilterSet::load().await;
    assert_eq!(f.matching(json!({ "shooter_ids": [f.id("Alex")] })).await, names(&["F1", "F2", "F7", "F8"]));
}

// DB-FLT-03
#[tokio::test]
async fn db_flt_03_empty_list() {
    let f = FilterSet::load().await;
    assert!(f.matching(json!({ "shooter_ids": [] })).await.is_empty());
}

// DB-FLT-04
#[tokio::test]
async fn db_flt_04_devices() {
    let f = FilterSet::load().await;
    assert_eq!(f.matching(json!({ "device_ids": [f.id("D3")] })).await, names(&["F4", "F5"]));
}

// DB-FLT-05
#[tokio::test]
async fn db_flt_05_device_categories() {
    let f = FilterSet::load().await;
    assert_eq!(f.matching(json!({ "device_categories": ["air pistol"] })).await, names(&["F1", "F3", "F6", "F7", "F8"]));
}

// DB-FLT-06
#[tokio::test]
async fn db_flt_06_calibers() {
    let f = FilterSet::load().await;
    assert_eq!(f.matching(json!({ "calibers": [".22 LR"] })).await, names(&["F4", "F5"]));
}

// DB-FLT-07
#[tokio::test]
async fn db_flt_07_fields_are_combined_with_and() {
    let f = FilterSet::load().await;
    assert!(f.matching(json!({ "device_categories": ["air pistol"], "device_ids": [f.id("D2")] })).await.is_empty());
}

// DB-FLT-08
#[tokio::test]
async fn db_flt_08_target_types() {
    let f = FilterSet::load().await;
    assert_eq!(f.matching(json!({ "target_type_ids": [f.id("TT-2")] })).await, names(&["F4", "F5"]));
}

// DB-FLT-09
#[tokio::test]
async fn db_flt_09_distance_from() {
    let f = FilterSet::load().await;
    assert_eq!(f.matching(json!({ "distance_m": { "min": 20 } })).await, names(&["F4", "F5"]));
}

// DB-FLT-10
#[tokio::test]
async fn db_flt_10_distance_bounds_are_inclusive() {
    let f = FilterSet::load().await;
    assert_eq!(f.matching(json!({ "distance_m": { "min": 10, "max": 10 } })).await, names(&["F1", "F2", "F3", "F6", "F7", "F8"]));
}

// DB-FLT-11
#[tokio::test]
async fn db_flt_11_time_range_inclusive() {
    let f = FilterSet::load().await;
    let from = f.h.session(f.id("F3")).await["saved_at"].as_i64().unwrap();
    let to = f.h.session(f.id("F5")).await["saved_at"].as_i64().unwrap();
    assert_eq!(f.matching(json!({ "saved_at": { "from": from, "to": to } })).await, names(&["F3", "F4", "F5"]));
}

// DB-FLT-12
#[tokio::test]
async fn db_flt_12_single_handed() {
    let f = FilterSet::load().await;
    assert_eq!(f.matching(json!({ "single_handed": true })).await, names(&["F1", "F3", "F5", "F6"]));
}

// DB-FLT-13
#[tokio::test]
async fn db_flt_13_supported() {
    let f = FilterSet::load().await;
    assert_eq!(f.matching(json!({ "supported": true })).await, names(&["F2"]));
}

// DB-FLT-14
#[tokio::test]
async fn db_flt_14_shot_count() {
    let f = FilterSet::load().await;
    assert_eq!(f.matching(json!({ "shot_count": { "min": 10, "max": 20 } })).await, names(&["F2", "F3"]));
}

// DB-FLT-15
#[tokio::test]
async fn db_flt_15_without_retired() {
    let f = FilterSet::load().await;
    assert_eq!(f.matching(json!({ "include_retired": false })).await, names(&["F1", "F2", "F3", "F4", "F5"]));
}

// DB-FLT-16
#[tokio::test]
async fn db_flt_16_session_ids() {
    let f = FilterSet::load().await;
    assert_eq!(f.matching(json!({ "session_ids": [f.id("F2"), f.id("F4")] })).await, names(&["F2", "F4"]));
}

// DB-FLT-17
#[tokio::test]
async fn db_flt_17_unknown_id() {
    let f = FilterSet::load().await;
    assert!(f.matching(json!({ "session_ids": [999999] })).await.is_empty());
}

// DB-FLT-18
#[tokio::test]
async fn db_flt_18_two_fields() {
    let f = FilterSet::load().await;
    assert_eq!(f.matching(json!({ "shooter_ids": [f.id("Alex")], "supported": true })).await, names(&["F2"]));
}

// DB-FLT-19
#[tokio::test]
async fn db_flt_19_sql_in_a_value() {
    let f = FilterSet::load().await;
    assert!(f.matching(json!({ "device_categories": ["x' OR '1'='1"] })).await.is_empty());
}

// DB-FLT-20
#[tokio::test]
async fn db_flt_20_default_order() {
    let f = FilterSet::load().await;
    let (order, _) = f.read(json!({ "filter": {} })).await;
    assert_eq!(order, names(&["F8", "F7", "F6", "F5", "F4", "F3", "F2", "F1"]));
}

// DB-FLT-21
#[tokio::test]
async fn db_flt_21_order_by_score() {
    let f = FilterSet::load().await;
    let (order, _) = f.read(json!({ "filter": {}, "order_by": { "field": "score", "direction": "asc" } })).await;
    assert_eq!(order, names(&["F4", "F8", "F1", "F7", "F6", "F2", "F3", "F5"]));
}

// DB-FLT-22
#[tokio::test]
async fn db_flt_22_order_by_shot_count() {
    let f = FilterSet::load().await;
    let (order, _) = f.read(json!({ "filter": {}, "order_by": { "field": "shot_count", "direction": "desc" } })).await;
    assert_eq!(order[..3], names(&["F5", "F3", "F2"]));
}

// DB-FLT-23
#[tokio::test]
async fn db_flt_23_order_by_a_field_that_isn_t_allowed() {
    let f = FilterSet::load().await;
    let code = f.h.err("read_sessions", json!({ "filter": {}, "order_by": { "field": "notes", "direction": "asc" } })).await;
    assert_eq!(code, "validation_failed");
}

// DB-FLT-24
#[tokio::test]
async fn db_flt_24_paging() {
    let f = FilterSet::load().await;
    let (order, total) = f.read(json!({ "filter": {}, "order_by": { "field": "id", "direction": "asc" }, "limit": 3, "offset": 3 })).await;
    assert_eq!(order, names(&["F4", "F5", "F6"]));
    assert_eq!(total, 8);
}

// DB-FLT-25
#[tokio::test]
async fn db_flt_25_page_after_the_end() {
    let f = FilterSet::load().await;
    let (order, total) = f.read(json!({ "filter": {}, "offset": 10 })).await;
    assert!(order.is_empty());
    assert_eq!(total, 8);
}
