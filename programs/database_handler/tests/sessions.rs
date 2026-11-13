// doc/testing/database_handler.md#sessions-create-read-delete-integration

mod common;

use common::*;
use serde_json::{json, Value};
use targetscope_testkit::fixtures;

fn paper_bytes() -> Vec<u8> {
    std::fs::read(fixtures::testdata("images/paper.jpg")).unwrap()
}

fn shot_on(paper_no: i64, angle_turns: f64, radius_mm: f64) -> Value {
    json!({ "angle_turns": angle_turns, "radius_mm": radius_mm, "paper_no": paper_no })
}

// DB-SES-01
#[tokio::test]
async fn db_ses_01_create_with_one_paper_and_image() {
    let h = Handler::start().await;
    let ids = h.refs().await;
    let staged = h.stage("paper.jpg");
    let payload = h.cs_ref(ids, Some(&staged));
    let r = h.ok("create_session", payload.clone()).await;
    let (id, uuid) = (r["id"].as_i64().unwrap(), r["uuid"].as_str().unwrap().to_string());
    assert!(is_uuid_v4(&uuid));
    let path = format!("sessions/{uuid}_1.jpg");
    assert_eq!(r["images"], json!([{ "paper_no": 1, "image_path": path }]));

    let s = h.session(id).await;
    for field in ["shooter_id", "device_id", "target_type_id", "distance_m", "single_handed", "supported", "shot_count",
        "paper_count", "score", "grouping_score", "duration_s", "notes"]
    {
        assert_eq!(s[field], payload[field], "{field}");
    }
    assert!((s["saved_at"].as_i64().unwrap() - now_s()).abs() <= 2);
    let db = h.db();
    let shots: Vec<(i64, i64, f64, f64)> = db
        .prepare("SELECT shot_no, paper_no, angle_turns, radius_mm FROM shots WHERE session_id = ?1 ORDER BY shot_no")
        .unwrap()
        .query_map([id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert_eq!(shots.len(), 5);
    for (i, ((no, paper, a, rad), expected)) in shots.iter().zip(fixtures::s_ref_shots()).enumerate() {
        assert_eq!(*no, i as i64 + 1);
        assert_eq!(*paper, 1);
        assert_eq!(*a, expected["angle_turns"].as_f64().unwrap());
        assert_eq!(*rad, expected["radius_mm"].as_f64().unwrap());
    }
    assert_eq!(h.count("SELECT count(*) FROM session_images"), 1);
    assert!(!h.dir.staging_dir.join(&staged).exists());
    assert_eq!(std::fs::read(h.image(&path)).unwrap(), paper_bytes());
    assert_eq!(h.device(ids.device_id).await["shot_count"], 5);
}

// DB-SES-02
#[tokio::test]
async fn db_ses_02_several_papers_some_without_image() {
    let h = Handler::start().await;
    let ids = h.refs().await;
    let (a, b) = (h.stage("paper.jpg"), h.stage("paper.jpg"));
    let mut p = h.cs_ref(ids, None);
    p["paper_count"] = json!(3);
    p["shots"] = json!([shot_on(1, 0.1, 1.5), shot_on(2, 0.2, 2.5), shot_on(3, 0.3, 3.5)]);
    p["shot_count"] = json!(3);
    p["images"] = json!([{ "paper_no": 1, "staged_file": a }, { "paper_no": 3, "staged_file": b }]);
    let r = h.ok("create_session", p).await;
    let uuid = r["uuid"].as_str().unwrap();
    let papers: Vec<i64> = h
        .db()
        .prepare("SELECT paper_no FROM session_images ORDER BY paper_no")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert_eq!(papers, [1, 3]);
    assert_eq!(files_in(&h.dir.image_dir.join("sessions")), [format!("{uuid}_1.jpg"), format!("{uuid}_3.jpg")]);
}

// DB-SES-03
#[tokio::test]
async fn db_ses_03_no_images() {
    let h = Handler::start().await;
    let ids = h.refs().await;
    let r = h.ok("create_session", h.cs_ref(ids, None)).await;
    assert_eq!(r["images"], json!([]));
}

// DB-SES-04
#[tokio::test]
async fn db_ses_04_png_image() {
    let h = Handler::start().await;
    let ids = h.refs().await;
    let staged = h.stage("device.png");
    let r = h.ok("create_session", h.cs_ref(ids, Some(&staged))).await;
    assert_eq!(r["images"][0]["image_path"], format!("sessions/{}_1.png", r["uuid"].as_str().unwrap()));
}

// DB-SES-05
#[tokio::test]
async fn db_ses_05_invalid_payloads() {
    let h = Handler::start().await;
    let ids = h.refs().await;
    let staged = h.stage("paper.jpg");
    let base = h.cs_ref(ids, Some(&staged));
    let mut cases: Vec<(&str, Value)> = vec![
        ("no shots", with(with(base.clone(), "shots", json!([])), "shot_count", json!(0))),
        ("shot_count 4", with(base.clone(), "shot_count", json!(4))),
        ("paper_count 0", with(base.clone(), "paper_count", json!(0))),
        ("notes 10 001", with(base.clone(), "notes", json!("x".repeat(10_001)))),
        ("unknown device", with(base.clone(), "device_id", json!(999))),
        ("unknown target type", with(base.clone(), "target_type_id", json!(999))),
        ("unknown shooter", with(base.clone(), "shooter_id", json!(999))),
        ("distance -1", with(base.clone(), "distance_m", json!(-1))),
        ("duration -1", with(base.clone(), "duration_s", json!(-1))),
        ("image on paper 2", with(base.clone(), "images", json!([{ "paper_no": 2, "staged_file": staged }]))),
        (
            "two images for paper 1",
            with(base.clone(), "images", json!([{ "paper_no": 1, "staged_file": staged }, { "paper_no": 1, "staged_file": staged }])),
        ),
    ];
    for (name, shot) in [
        ("shot on paper 0", shot_on(0, 0.1, 1.5)),
        ("shot on paper 2", shot_on(2, 0.1, 1.5)),
        ("angle_turns 1.0", shot_on(1, 1.0, 1.5)),
        ("radius -1", shot_on(1, 0.1, -1.0)),
    ] {
        let mut p = base.clone();
        p["shots"][0] = shot;
        cases.push((name, p));
    }
    let mut probe = h.probe().await;
    for (name, payload) in cases {
        assert_eq!(h.err("create_session", payload).await, "validation_failed", "{name}");
    }
    assert_eq!(h.count("SELECT count(*) FROM sessions"), 0);
    assert!(h.dir.staging_dir.join(&staged).exists());
    assert_eq!(h.device(ids.device_id).await["shot_count"], 0);
    probe.expect_none(NO_EVENT_WAIT).await;
}

// DB-SES-06
#[tokio::test]
async fn db_ses_06_longest_notes() {
    let h = Handler::start().await;
    let ids = h.refs().await;
    let notes = "ä".repeat(10_000);
    let id = h.ok("create_session", with(h.cs_ref(ids, None), "notes", json!(notes))).await["id"].as_i64().unwrap();
    assert_eq!(h.session(id).await["notes"], notes.as_str());
}

// DB-SES-07
#[tokio::test]
async fn db_ses_07_staged_image_missing() {
    let h = Handler::start().await;
    let ids = h.refs().await;
    assert_eq!(h.err("create_session", h.cs_ref(ids, Some("upload_missing.jpg"))).await, "not_found");
}

// DB-SES-08
#[tokio::test]
async fn db_ses_08_staged_name_with_a_path() {
    let h = Handler::start().await;
    let ids = h.refs().await;
    std::fs::copy(fixtures::testdata("images/paper.jpg"), h.dir.data_dir.join("paper.jpg")).unwrap();
    std::fs::create_dir_all(h.dir.staging_dir.join("sub")).unwrap();
    std::fs::copy(fixtures::testdata("images/paper.jpg"), h.dir.staging_dir.join("sub/paper.jpg")).unwrap();
    for name in ["../paper.jpg", "sub/paper.jpg"] {
        assert_eq!(h.err("create_session", h.cs_ref(ids, Some(name))).await, "validation_failed", "{name}");
    }
}

// DB-SES-09
#[tokio::test]
async fn db_ses_09_staged_file_isn_t_an_image() {
    let h = Handler::start().await;
    let ids = h.refs().await;
    let staged = h.stage("not_an_image.jpg");
    assert_eq!(h.err("create_session", h.cs_ref(ids, Some(&staged))).await, "validation_failed");
    assert!(h.dir.staging_dir.join(&staged).exists());
}

// DB-SES-10
#[tokio::test]
#[cfg_attr(not(feature = "failpoints"), ignore = "needs --features failpoints")]
async fn db_ses_10_failure_before_commit() {
    let mut h = Handler::start().await;
    let ids = h.refs().await;
    h.restart_with(&[("FAILPOINTS", "db.before_commit=return")]).await;
    let staged = h.stage("paper.jpg");
    let mut probe = h.probe().await;
    assert_eq!(h.err("create_session", h.cs_ref(ids, Some(&staged))).await, "database_error");
    assert_eq!(h.count("SELECT count(*) FROM sessions"), 0);
    assert_eq!(std::fs::read(h.dir.staging_dir.join(&staged)).unwrap(), paper_bytes());
    assert!(files_in(&h.dir.image_dir.join("sessions")).is_empty());
    assert_eq!(h.count(&format!("SELECT shot_count FROM devices WHERE id = {}", ids.device_id)), 0);
    assert_eq!(h.count("SELECT count(*) FROM processed_requests WHERE type = 'create_session'"), 0);
    probe.expect_none(NO_EVENT_WAIT).await;
}

// DB-SES-11
#[tokio::test]
async fn db_ses_11_read_one_session() {
    let h = Handler::start().await;
    let ids = h.refs().await;
    let (a, b) = (h.stage("paper.jpg"), h.stage("paper.jpg"));
    let mut p = h.cs_ref(ids, None);
    p["paper_count"] = json!(2);
    p["shots"] = json!([shot_on(2, 0.5, 5.5), shot_on(1, 0.25, 2.5), shot_on(2, 0.75, 7.5)]);
    p["shot_count"] = json!(3);
    p["images"] = json!([{ "paper_no": 2, "staged_file": b }, { "paper_no": 1, "staged_file": a }]);
    let id = h.ok("create_session", p.clone()).await["id"].as_i64().unwrap();
    let r = h.ok("read_sessions", json!({ "filter": { "session_ids": [id] }, "include_shots": true })).await;
    assert_eq!(r["total"], 1);
    let s = &r["sessions"][0];
    assert_eq!(s["shots"], p["shots"]);
    let papers: Vec<i64> = s["images"].as_array().unwrap().iter().map(|i| i["paper_no"].as_i64().unwrap()).collect();
    assert_eq!(papers, [1, 2]);
    for field in ["id", "uuid", "saved_at", "score", "grouping_score", "distance_m", "target_type_id", "shooter_id", "device_id",
        "single_handed", "supported", "shot_count", "duration_s", "notes", "paper_count"]
    {
        assert!(s.get(field).is_some(), "{field}");
    }
}

// DB-SES-12
#[tokio::test]
async fn db_ses_12_without_shots() {
    let h = Handler::start().await;
    let ids = h.refs().await;
    let id = h.create_session(ids).await["id"].as_i64().unwrap();
    let r = h.ok("read_sessions", json!({ "filter": { "session_ids": [id] }, "include_shots": false })).await;
    let s = &r["sessions"][0];
    assert_eq!(s["shots"], json!([]));
    assert_eq!(s["shot_count"], 5);
    assert_eq!(s["images"].as_array().unwrap().len(), 1);
}

// DB-SES-13
#[tokio::test]
async fn db_ses_13_delete() {
    let h = Handler::start().await;
    let ids = h.refs().await;
    let r = h.create_session(ids).await;
    let id = r["id"].as_i64().unwrap();
    let path = r["images"][0]["image_path"].as_str().unwrap().to_string();
    let mut probe = h.probe().await;
    assert_eq!(h.ok("delete_session", json!({ "id": id })).await["id"], id);
    for table in ["sessions", "shots", "session_images"] {
        assert_eq!(h.count(&format!("SELECT count(*) FROM {table}")), 0, "{table}");
    }
    assert!(!h.image(&path).exists());
    assert_eq!(h.device(ids.device_id).await["shot_count"], 5);
    assert_eq!(changes(&mut probe, 1).await, [format!("sessions/{id}/deleted")]);
}

// DB-SES-14
#[tokio::test]
async fn db_ses_14_delete_unknown() {
    let h = Handler::start().await;
    assert_eq!(h.err("delete_session", json!({ "id": 999 })).await, "not_found");
}

// DB-SES-15
#[tokio::test]
async fn db_ses_15_delete_with_a_missing_image_file() {
    let h = Handler::start().await;
    let ids = h.refs().await;
    let r = h.create_session(ids).await;
    std::fs::remove_file(h.image(r["images"][0]["image_path"].as_str().unwrap())).unwrap();
    h.ok("delete_session", json!({ "id": r["id"] })).await;
    assert_eq!(h.count("SELECT count(*) FROM sessions"), 0);
}

// DB-SES-16
#[tokio::test]
#[cfg_attr(not(feature = "failpoints"), ignore = "needs --features failpoints")]
async fn db_ses_16_failure_before_commit_when_deleting() {
    let mut h = Handler::start().await;
    let ids = h.refs().await;
    let r = h.create_session(ids).await;
    let path = r["images"][0]["image_path"].as_str().unwrap().to_string();
    h.restart_with(&[("FAILPOINTS", "db.before_commit=return")]).await;
    assert_eq!(h.err("delete_session", json!({ "id": r["id"] })).await, "database_error");
    assert_eq!(h.count("SELECT count(*) FROM sessions"), 1);
    assert_eq!(h.count("SELECT count(*) FROM shots"), 5);
    assert!(h.image(&path).exists());
}
