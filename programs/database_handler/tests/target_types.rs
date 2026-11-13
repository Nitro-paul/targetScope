// doc/testing/database_handler.md#target-types-integration

mod common;

use common::*;
use serde_json::{json, Value};
use targetscope_testkit::fixtures;

fn tt_ref() -> Value {
    serde_json::to_value(fixtures::tt_ref()).unwrap()
}

async fn target_type(h: &Handler, id: i64) -> Value {
    let r = h.ok("read_target_types", json!({ "filter": { "conditions": [{ "field": "id", "op": "eq", "value": id }] } })).await;
    r["target_types"][0].clone()
}

fn rings(t: &Value) -> Vec<(i64, f64)> {
    t["rings"].as_array().unwrap().iter().map(|r| (r["score"].as_i64().unwrap(), r["radius_mm"].as_f64().unwrap())).collect()
}

// LIB-TGT-01
const TT_REF_RINGS: [(i64, f64); 12] =
    [(12, 5.0), (11, 10.0), (10, 15.0), (9, 20.0), (8, 25.0), (7, 30.0), (6, 40.0), (5, 50.0), (4, 60.0), (3, 70.0), (2, 80.0), (1, 90.0)];

fn assert_rings(t: &Value, expected: &[(i64, f64)]) {
    let got = rings(t);
    assert_eq!(got.len(), expected.len(), "{got:?}");
    for ((s, r), (es, er)) in got.iter().zip(expected) {
        assert_eq!(s, es);
        assert!((r - er).abs() < 1e-9, "ring {s}: {r} != {er}");
    }
}

// DB-TGT-01
#[tokio::test]
async fn db_tgt_01_create_and_read() {
    let h = Handler::start().await;
    let id = h.ok("save_target_type", tt_ref()).await["id"].as_i64().unwrap();
    let t = target_type(&h, id).await;
    assert!(is_uuid_v4(t["uuid"].as_str().unwrap()));
    assert_rings(&t, &TT_REF_RINGS);
    assert_eq!(t["locked"], false);
    assert_eq!(t["retired"], false);
}

// DB-TGT-02
#[tokio::test]
async fn db_tgt_02_rings_in_the_payload_are_ignored() {
    let h = Handler::start().await;
    let p = with(
        tt_ref(),
        "rings",
        json!([{ "score": 3, "radius_mm": 1.0 }, { "score": 2, "radius_mm": 2.0 }, { "score": 1, "radius_mm": 3.0 }]),
    );
    let id = h.ok("save_target_type", p).await["id"].as_i64().unwrap();
    assert_rings(&target_type(&h, id).await, &TT_REF_RINGS);
}

// DB-TGT-03
#[tokio::test]
async fn db_tgt_03_invalid_target_types() {
    let h = Handler::start().await;
    // LIB-TGT-07
    let changes = [
        ("inner_white_score", json!(7)),
        ("black_score", json!(1)),
        ("outer_score", json!(0)),
        ("inner_white_radius_mm", json!(0.0)),
        ("inner_white_radius_mm", json!(30.0)),
        ("black_radius_mm", json!(90.0)),
        ("center_x_mm", json!(89.0)),
        ("center_x_mm", json!(111.0)),
        ("center_y_mm", json!(89.0)),
        ("center_y_mm", json!(111.0)),
        ("paper_width_mm", json!(0.0)),
    ];
    for (field, value) in changes {
        assert_eq!(h.err("save_target_type", with(tt_ref(), field, value.clone())).await, "validation_failed", "{field} {value}");
    }
    assert_eq!(h.count("SELECT count(*) FROM target_types"), 0);
}

// DB-TGT-04
#[tokio::test]
async fn db_tgt_04_name_exists_other_case() {
    let h = Handler::start().await;
    h.ok("save_target_type", tt_ref()).await;
    assert_eq!(h.err("save_target_type", with(tt_ref(), "name", json!("tt-ref"))).await, "validation_failed");
}

// DB-TGT-05
#[tokio::test]
async fn db_tgt_05_locked_target_type() {
    let h = Handler::start().await;
    let ids = h.refs().await;
    h.create_session(ids).await;
    let before = target_type(&h, ids.target_type_id).await;
    let base = with(tt_ref(), "id", json!(ids.target_type_id));
    let fields = [
        ("paper_width_mm", json!(210.0)),
        ("paper_height_mm", json!(210.0)),
        ("center_x_mm", json!(101.0)),
        ("center_y_mm", json!(101.0)),
        ("inner_white_radius_mm", json!(4.0)),
        ("inner_white_score", json!(11)),
        ("black_radius_mm", json!(31.0)),
        ("black_score", json!(6)),
        ("outer_radius_mm", json!(89.0)),
        ("outer_score", json!(2)),
    ];
    for (field, value) in fields {
        assert_eq!(h.err("save_target_type", with(base.clone(), field, value)).await, "in_use", "{field}");
    }
    assert_eq!(target_type(&h, ids.target_type_id).await, before);
}

// DB-TGT-06
#[tokio::test]
async fn db_tgt_06_locked_target_type_free_fields() {
    let h = Handler::start().await;
    let ids = h.refs().await;
    h.create_session(ids).await;
    let p = with(with(with(tt_ref(), "id", json!(ids.target_type_id)), "name", json!("TT-NEW")), "retired", json!(true));
    h.ok("save_target_type", p).await;
    let t = target_type(&h, ids.target_type_id).await;
    assert_eq!(t["name"], "TT-NEW");
    assert_eq!(t["retired"], true);
    assert_eq!(t["locked"], true);
}

// DB-TGT-07
#[tokio::test]
async fn db_tgt_07_unlocked_target_type() {
    let h = Handler::start().await;
    let id = h.ok("save_target_type", tt_ref()).await["id"].as_i64().unwrap();
    h.ok("save_target_type", with(with(tt_ref(), "id", json!(id)), "outer_radius_mm", json!(70.0))).await;
    let t = target_type(&h, id).await;
    assert_eq!(t["outer_radius_mm"], 70.0);
    let r = rings(&t);
    assert_eq!(r.last().unwrap(), &(1, 70.0));
    assert_eq!(r[5], (7, 30.0));
}

// DB-TGT-08
#[tokio::test]
async fn db_tgt_08_delete() {
    let h = Handler::start().await;
    let ids = h.refs().await;
    let unused = h.ok("save_target_type", with(tt_ref(), "name", json!("TT-2"))).await["id"].as_i64().unwrap();
    assert_eq!(h.ok("delete_target_type", json!({ "id": unused })).await["id"], unused);
    h.create_session(ids).await;
    assert_eq!(h.err("delete_target_type", json!({ "id": ids.target_type_id })).await, "in_use");
}

// DB-TGT-09
#[tokio::test]
async fn db_tgt_09_unknown_id() {
    let h = Handler::start().await;
    assert_eq!(h.err("save_target_type", with(tt_ref(), "id", json!(999))).await, "not_found");
}
