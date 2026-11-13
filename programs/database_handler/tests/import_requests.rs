// doc/testing/database_handler.md#import-requests-integration

mod common;

use common::*;
use serde_json::{json, Value};
use targetscope_testkit::fixtures::{self, ReferenceIds};

const U1: &str = "11111111-1111-4111-8111-111111111111";
const U2: &str = "22222222-2222-4222-8222-222222222222";
const U3: &str = "33333333-3333-4333-8333-333333333333";
const SAVED_AT_2025_05_01_10H_UTC: i64 = 1746093600;

fn tt_ref(uuid: &str) -> Value {
    with(serde_json::to_value(fixtures::tt_ref()).unwrap(), "uuid", json!(uuid))
}

fn device(uuid: &str, shot_count: i64) -> Value {
    with(with(serde_json::to_value(fixtures::dev_ref()).unwrap(), "uuid", json!(uuid)), "shot_count", json!(shot_count))
}

fn imported_session(ids: ReferenceIds, uuid: &str, images: Value) -> Value {
    let mut p = cs_ref(ids, None);
    p["uuid"] = json!(uuid);
    p["saved_at"] = json!(SAVED_AT_2025_05_01_10H_UTC);
    p["images"] = images;
    p
}

fn created(r: &Value) -> (i64, bool) {
    (r["id"].as_i64().unwrap(), r["created"].as_bool().unwrap())
}

// DB-IMP-01
#[tokio::test]
async fn db_imp_01_new_shooter() {
    let h = Handler::start().await;
    let (id, new) = created(&h.ok("import_shooter", json!({ "uuid": U1, "name": "Alex" })).await);
    assert!(new);
    let s = h.ok("read_shooters", json!({})).await["shooters"][0].clone();
    assert_eq!(s["id"], id);
    assert_eq!(s["uuid"], U1);
}

// DB-IMP-02
#[tokio::test]
async fn db_imp_02_shooter_uuid_exists() {
    let h = Handler::start().await;
    let (id, _) = created(&h.ok("import_shooter", json!({ "uuid": U1, "name": "Alex" })).await);
    assert_eq!(created(&h.ok("import_shooter", json!({ "uuid": U1, "name": "Other" })).await), (id, false));
    let all = h.ok("read_shooters", json!({})).await["shooters"].clone();
    assert_eq!(all.as_array().unwrap().len(), 1);
    assert_eq!(all[0]["name"], "Alex");
}

// DB-IMP-03
#[tokio::test]
async fn db_imp_03_shooter_name_exists() {
    let h = Handler::start().await;
    let id = h.ok("save_shooter", json!({ "name": "Alex" })).await["id"].as_i64().unwrap();
    assert_eq!(created(&h.ok("import_shooter", json!({ "uuid": U1, "name": "ALEX" })).await), (id, false));
    assert_eq!(h.count("SELECT count(*) FROM shooters"), 1);
}

// DB-IMP-04
#[tokio::test]
async fn db_imp_04_new_target_type() {
    let h = Handler::start().await;
    let (id, new) = created(&h.ok("import_target_type", tt_ref(U1)).await);
    assert!(new);
    let uuid: String = h.db().query_row("SELECT uuid FROM target_types WHERE id = ?1", [id], |r| r.get(0)).unwrap();
    assert_eq!(uuid, U1);
}

// DB-IMP-05
#[tokio::test]
async fn db_imp_05_target_type_uuid_exists() {
    let h = Handler::start().await;
    let (id, _) = created(&h.ok("import_target_type", tt_ref(U1)).await);
    let other = with(tt_ref(U1), "outer_radius_mm", json!(80.0));
    assert_eq!(created(&h.ok("import_target_type", other).await), (id, false));
    let r: f64 = h.db().query_row("SELECT outer_radius_mm FROM target_types WHERE id = ?1", [id], |r| r.get(0)).unwrap();
    assert_eq!(r, 90.0);
}

// DB-IMP-06
#[tokio::test]
async fn db_imp_06_target_type_name_exists() {
    let h = Handler::start().await;
    h.ok("save_target_type", serde_json::to_value(fixtures::tt_ref()).unwrap()).await;
    for (uuid, expected) in [(U1, "TT-REF (imported)"), (U2, "TT-REF (imported 2)")] {
        let (id, new) = created(&h.ok("import_target_type", tt_ref(uuid)).await);
        assert!(new);
        let name: String = h.db().query_row("SELECT name FROM target_types WHERE id = ?1", [id], |r| r.get(0)).unwrap();
        assert_eq!(name, expected);
    }
}

// DB-IMP-07
#[tokio::test]
async fn db_imp_07_new_device_with_image() {
    let h = Handler::start().await;
    let staged = h.stage("paper.jpg");
    let (id, new) = created(&h.ok("import_device", with(device(U1, 123), "staged_file", json!(staged))).await);
    assert!(new);
    let d = h.device(id).await;
    assert_eq!(d["shot_count"], 123);
    let path = d["image_path"].as_str().unwrap();
    let hex = path.strip_prefix(&format!("devices/{id}_")).and_then(|r| r.strip_suffix(".jpg")).unwrap();
    assert!(hex.len() == 8 && hex.chars().all(|c| c.is_ascii_hexdigit()), "{path}");
    assert!(h.image(path).exists());
    assert!(!h.dir.staging_dir.join(&staged).exists());
}

// DB-IMP-08
#[tokio::test]
async fn db_imp_08_device_uuid_exists() {
    let h = Handler::start().await;
    let (id, _) = created(&h.ok("import_device", device(U1, 123)).await);
    let before = h.device(id).await;
    let staged = h.stage("paper.jpg");
    let again = with(with(device(U1, 999), "staged_file", json!(staged)), "name", json!("Other"));
    assert_eq!(created(&h.ok("import_device", again).await), (id, false));
    assert_eq!(h.device(id).await, before);
    assert!(h.dir.staging_dir.join(&staged).exists());
}

// DB-IMP-09
#[tokio::test]
async fn db_imp_09_new_session() {
    let h = Handler::start().await;
    let ids = h.refs().await;
    let (a, b) = (h.stage("paper.jpg"), h.stage("device.png"));
    let mut p = imported_session(ids, U1, json!([{ "paper_no": 1, "staged_file": a }, { "paper_no": 2, "staged_file": b }]));
    p["paper_count"] = json!(2);
    let (id, new) = created(&h.ok("import_session", p).await);
    assert!(new);
    let s = h.session(id).await;
    assert_eq!(s["uuid"], U1);
    assert_eq!(s["saved_at"], SAVED_AT_2025_05_01_10H_UTC);
    let paths: Vec<&str> = s["images"].as_array().unwrap().iter().map(|i| i["image_path"].as_str().unwrap()).collect();
    assert_eq!(paths, [format!("sessions/{U1}_1.jpg"), format!("sessions/{U1}_2.png")]);
    assert_eq!(h.device(ids.device_id).await["shot_count"], 0);
}

// DB-IMP-10
#[tokio::test]
async fn db_imp_10_session_uuid_exists() {
    let h = Handler::start().await;
    let ids = h.refs().await;
    let (id, _) = created(&h.ok("import_session", imported_session(ids, U1, json!([]))).await);
    let staged = h.stage("paper.jpg");
    let again = with(imported_session(ids, U1, json!([{ "paper_no": 1, "staged_file": staged }])), "score", json!(1));
    assert_eq!(created(&h.ok("import_session", again).await), (id, false));
    assert_eq!(h.session(id).await["score"], 38);
    assert_eq!(h.count("SELECT count(*) FROM session_images"), 0);
    assert!(h.dir.staging_dir.join(&staged).exists());
}

// DB-IMP-11
#[tokio::test]
async fn db_imp_11_invalid_session() {
    let h = Handler::start().await;
    let ids = h.refs().await;
    let base = imported_session(ids, U1, json!([]));
    let mut cases = vec![
        with(with(base.clone(), "shots", json!([])), "shot_count", json!(0)),
        with(base.clone(), "shot_count", json!(4)),
        with(base.clone(), "paper_count", json!(0)),
        with(base.clone(), "notes", json!("x".repeat(10_001))),
        with(base.clone(), "device_id", json!(999)),
        with(base.clone(), "target_type_id", json!(999)),
        with(base.clone(), "shooter_id", json!(999)),
        with(base.clone(), "distance_m", json!(-1)),
        with(base.clone(), "duration_s", json!(-1)),
    ];
    for shot in [
        json!({ "angle_turns": 0.1, "radius_mm": 1.5, "paper_no": 0 }),
        json!({ "angle_turns": 0.1, "radius_mm": 1.5, "paper_no": 2 }),
        json!({ "angle_turns": 1.0, "radius_mm": 1.5, "paper_no": 1 }),
        json!({ "angle_turns": 0.1, "radius_mm": -1.0, "paper_no": 1 }),
    ] {
        let mut p = base.clone();
        p["shots"][0] = shot;
        cases.push(p);
    }
    for (i, p) in cases.into_iter().enumerate() {
        assert_eq!(h.err("import_session", p).await, "validation_failed", "case {i}");
    }
    assert_eq!(h.count("SELECT count(*) FROM sessions"), 0);
}

// DB-IMP-12
#[tokio::test]
async fn db_imp_12_invalid_uuid() {
    let h = Handler::start().await;
    let ids = h.refs().await;
    let bad = "not-a-uuid";
    for (request, payload) in [
        ("import_shooter", json!({ "uuid": bad, "name": "Alex" })),
        ("import_target_type", tt_ref(bad)),
        ("import_device", device(bad, 1)),
        ("import_session", imported_session(ids, bad, json!([]))),
    ] {
        assert_eq!(h.err(request, payload).await, "validation_failed", "{request}");
    }
}

// DB-IMP-13
#[tokio::test]
async fn db_imp_13_missing_saved_at() {
    let h = Handler::start().await;
    let ids = h.refs().await;
    assert_eq!(h.err("import_session", without(imported_session(ids, U3, json!([])), "saved_at")).await, "validation_failed");
}
