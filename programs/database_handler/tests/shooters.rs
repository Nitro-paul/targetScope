// doc/testing/database_handler.md#shooters-integration

mod common;

use common::*;
use serde_json::json;

async fn shooters(h: &Handler) -> Vec<serde_json::Value> {
    h.ok("read_shooters", json!({})).await["shooters"].as_array().unwrap().clone()
}

// DB-SHO-01
#[tokio::test]
async fn db_sho_01_create() {
    let h = Handler::start().await;
    let mut probe = h.probe().await;
    let id = h.ok("save_shooter", json!({ "name": "  Alex  " })).await["id"].as_i64().unwrap();
    let all = shooters(&h).await;
    assert_eq!(all.len(), 1);
    assert_eq!(all[0]["id"], id);
    assert_eq!(all[0]["name"], "Alex");
    assert!(is_uuid_v4(all[0]["uuid"].as_str().unwrap()));
    assert_eq!(all[0]["retired"], false);
    assert_eq!(changes(&mut probe, 1).await, [format!("shooters/{id}/created")]);
}

// DB-SHO-02
#[tokio::test]
async fn db_sho_02_empty_name() {
    let h = Handler::start().await;
    assert_eq!(h.err("save_shooter", json!({ "name": "" })).await, "validation_failed");
    assert_eq!(h.err("save_shooter", json!({ "name": "   " })).await, "validation_failed");
    assert!(shooters(&h).await.is_empty());
}

// DB-SHO-03
#[tokio::test]
async fn db_sho_03_name_exists_other_case() {
    let h = Handler::start().await;
    h.ok("save_shooter", json!({ "name": "Alex" })).await;
    assert_eq!(h.err("save_shooter", json!({ "name": "alex" })).await, "validation_failed");
    assert_eq!(shooters(&h).await.len(), 1);
}

// DB-SHO-04
#[tokio::test]
async fn db_sho_04_rename() {
    let h = Handler::start().await;
    let ids = h.refs().await;
    let session = h.create_session(ids).await["id"].as_i64().unwrap();
    let mut probe = h.probe().await;
    h.ok("save_shooter", json!({ "id": ids.shooter_id, "name": "Alexandra" })).await;
    assert_eq!(shooters(&h).await[0]["name"], "Alexandra");
    assert_eq!(h.session(session).await["shooter_id"], ids.shooter_id);
    assert_eq!(changes(&mut probe, 1).await, [format!("shooters/{}/updated", ids.shooter_id)]);
}

// DB-SHO-05
#[tokio::test]
async fn db_sho_05_retire() {
    let h = Handler::start().await;
    let ids = h.refs().await;
    let session = h.create_session(ids).await["id"].as_i64().unwrap();
    let before = h.session(session).await;
    h.ok("save_shooter", json!({ "id": ids.shooter_id, "name": "Alex", "retired": true })).await;
    let all = shooters(&h).await;
    assert_eq!(all.len(), 1);
    assert_eq!(all[0]["retired"], true);
    assert_eq!(h.session(session).await, before);
}

// DB-SHO-06
#[tokio::test]
async fn db_sho_06_unknown_id() {
    let h = Handler::start().await;
    assert_eq!(h.err("save_shooter", json!({ "id": 999, "name": "Alex" })).await, "not_found");
}

// DB-SHO-07
#[tokio::test]
async fn db_sho_07_delete_unused() {
    let h = Handler::start().await;
    let id = h.ok("save_shooter", json!({ "name": "Alex" })).await["id"].as_i64().unwrap();
    let mut probe = h.probe().await;
    assert_eq!(h.ok("delete_shooter", json!({ "id": id })).await["id"], id);
    assert!(shooters(&h).await.is_empty());
    assert_eq!(changes(&mut probe, 1).await, [format!("shooters/{id}/deleted")]);
}

// DB-SHO-08
#[tokio::test]
async fn db_sho_08_delete_used() {
    let h = Handler::start().await;
    let ids = h.refs().await;
    h.create_session(ids).await;
    assert_eq!(h.err("delete_shooter", json!({ "id": ids.shooter_id })).await, "in_use");
    assert_eq!(shooters(&h).await.len(), 1);
}

// DB-SHO-09
#[tokio::test]
async fn db_sho_09_filter() {
    let h = Handler::start().await;
    for name in ["Alex", "Bea", "Chris"] {
        h.ok("save_shooter", json!({ "name": name })).await;
    }
    let r = h
        .ok("read_shooters", json!({ "filter": { "conditions": [{ "field": "name", "op": "like", "value": "Al%" }] } }))
        .await;
    let names: Vec<&str> = r["shooters"].as_array().unwrap().iter().map(|s| s["name"].as_str().unwrap()).collect();
    assert_eq!(names, ["Alex"]);
}

// DB-SHO-10
#[tokio::test]
async fn db_sho_10_uuid_in_the_payload_is_ignored() {
    let h = Handler::start().await;
    let given = "0b3f1c2e-6c1a-4a8e-9d55-2f6b1d6c7a10";
    h.ok("save_shooter", json!({ "name": "Alex", "uuid": given })).await;
    let stored = shooters(&h).await[0]["uuid"].as_str().unwrap().to_string();
    assert_ne!(stored, given);
    assert!(is_uuid_v4(&stored));
}
