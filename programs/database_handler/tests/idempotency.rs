// doc/testing/database_handler.md#duplicate-protection-integration

mod common;

use std::path::Path;
use std::time::Duration;

use common::*;
use serde_json::{json, Value};
use targetscope_common::messages::{ResponseEnvelope, Status};
use targetscope_testkit::fixtures;
use uuid::Uuid;

fn fingerprint(h: &Handler) -> String {
    let db = h.db();
    let mut out = String::new();
    for table in ["settings", "shooters", "devices", "target_types", "sessions", "shots", "session_images"] {
        let mut stmt = db.prepare(&format!("SELECT * FROM {table} ORDER BY rowid")).unwrap();
        let n = stmt.column_count();
        let rows: Vec<String> = stmt
            .query_map([], |r| Ok((0..n).map(|i| format!("{:?}", r.get::<_, rusqlite::types::Value>(i).unwrap())).collect::<Vec<_>>().join("|")))
            .unwrap()
            .map(Result::unwrap)
            .collect();
        out.push_str(&format!("{table}: {rows:?}\n"));
    }
    out.push_str(&format!("files: {:?}", all_files(&h.dir.image_dir)));
    out
}

fn all_files(dir: &Path) -> Vec<String> {
    let mut v = Vec::new();
    for e in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        if e.path().is_dir() {
            v.extend(all_files(&e.path()).into_iter().map(|f| format!("{}/{f}", e.file_name().to_string_lossy())));
        } else {
            v.push(e.file_name().to_string_lossy().into_owned());
        }
    }
    v.sort();
    v
}

async fn send_twice(h: &Handler, r#type: &str, payload: Value) -> (ResponseEnvelope, ResponseEnvelope, String, String) {
    let id = Uuid::new_v4();
    let first = h.bus.send_with_id(P, id, r#type, payload.clone()).await;
    assert_eq!(first.status, Status::Ok, "{type}: {:?}", first.error);
    let after_first = fingerprint(h);
    let second = h.bus.send_with_id(P, id, r#type, payload).await;
    let after_second = fingerprint(h);
    let rows: Vec<(String, i64, String)> = h
        .db()
        .prepare("SELECT type, processed_at, response FROM processed_requests WHERE request_id = ?1")
        .unwrap()
        .query_map([id.to_string()], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert_eq!(rows.len(), 1, "{type}: processed_requests rows");
    assert_eq!(rows[0].0, r#type);
    assert!((rows[0].1 - now_s()).abs() <= 5);
    assert_eq!(serde_json::from_str::<Value>(&rows[0].2).unwrap(), first.payload.clone().unwrap(), "{type}: stored response");
    (first, second, after_first, after_second)
}

// DB-IDM-01
#[tokio::test]
async fn db_idm_01_same_request_twice() {
    let h = Handler::start().await;
    let ids = h.refs().await;
    let session = h.ok("create_session", cs_ref(ids, None)).await["id"].as_i64().unwrap();
    let new_id = |r: Value| r["id"].as_i64().unwrap();
    let shooter = new_id(h.ok("save_shooter", json!({ "name": "Unused" })).await);
    let device = new_id(h.ok("save_device", with(serde_json::to_value(fixtures::dev_ref()).unwrap(), "name", json!("Unused"))).await);
    let tt = with(serde_json::to_value(fixtures::tt_ref()).unwrap(), "name", json!("TT-UNUSED"));
    let target_type = new_id(h.ok("save_target_type", tt.clone()).await);
    h.ok("save_setting", json!({ "name": "own", "display_name": "own", "value": "x", "active": true })).await;
    let staged = h.stage("paper.jpg");
    let mut imported = cs_ref(ids, None);
    imported["uuid"] = json!("44444444-4444-4444-8444-444444444444");
    imported["saved_at"] = json!(1746093600);

    let cases: Vec<(&str, Value)> = vec![
        ("create_session", cs_ref(ids, None)),
        ("delete_session", json!({ "id": session })),
        ("save_shooter", json!({ "name": "Bea" })),
        ("delete_shooter", json!({ "id": shooter })),
        ("save_device", with(serde_json::to_value(fixtures::dev_ref()).unwrap(), "name", json!("Second"))),
        ("set_device_image", json!({ "device_id": ids.device_id, "staged_file": staged })),
        ("delete_device_image", json!({ "device_id": ids.device_id })),
        ("delete_device", json!({ "id": device })),
        ("save_target_type", with(tt.clone(), "name", json!("TT-NEW"))),
        ("delete_target_type", json!({ "id": target_type })),
        ("import_shooter", json!({ "uuid": "11111111-1111-4111-8111-111111111111", "name": "Imported" })),
        ("import_target_type", with(with(tt, "name", json!("TT-IMP")), "uuid", json!("22222222-2222-4222-8222-222222222222"))),
        (
            "import_device",
            with(with(serde_json::to_value(fixtures::dev_ref()).unwrap(), "name", json!("Imp")), "uuid", json!("33333333-3333-4333-8333-333333333333")),
        ),
        ("import_session", imported),
        ("save_setting", json!({ "name": "camera_fps", "display_name": "x", "value": "15", "active": true })),
        ("delete_setting", json!({ "name": "own" })),
        ("reset_settings", json!({ "names": ["camera_fps"] })),
        ("change_image_dir", json!({ "path": h.dir.data_dir.join("images2").display().to_string(), "move_images": false })),
    ];
    for (r#type, payload) in cases {
        let (first, second, after_first, after_second) = send_twice(&h, r#type, payload).await;
        assert_eq!(second.status, Status::Ok, "{type}: {:?}", second.error);
        assert_eq!(second.payload, first.payload, "{type}");
        assert_eq!(after_second, after_first, "{type}: effect twice");
    }
}

// DB-IDM-02
#[tokio::test]
#[cfg_attr(not(feature = "failpoints"), ignore = "needs --features failpoints")]
async fn db_idm_02_failed_requests_aren_t_recorded() {
    let mut h = Handler::start().await;
    let ids = h.refs().await;
    h.restart_with(&[("FAILPOINTS", "db.before_commit=return")]).await;
    let id = Uuid::new_v4();
    let r = h.bus.send_with_id(P, id, "create_session", cs_ref(ids, None)).await;
    assert_eq!(r.error.unwrap().code.as_str(), "database_error");
    h.restart().await;
    let r = h.bus.send_with_id(P, id, "create_session", cs_ref(ids, None)).await;
    assert_eq!(r.status, Status::Ok, "{:?}", r.error);
    assert_eq!(h.count("SELECT count(*) FROM sessions"), 1);
}

// DB-IDM-03
#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "failpoints"), ignore = "needs --features failpoints")]
async fn db_idm_03_crash_after_commit() {
    let mut h = Handler::start().await;
    let ids = h.refs().await;
    h.restart_with(&[("FAILPOINTS", "db.after_commit=return(abort)")]).await;
    let id = Uuid::new_v4();
    let waiting = {
        let bus = targetscope_testkit::TestBus::connect(&h.redis.url()).await;
        let payload = cs_ref(ids, None);
        tokio::spawn(async move {
            let stream = targetscope_common::messages::request_stream(P);
            let r = bus.bus.send_envelope(
                &stream,
                targetscope_common::messages::RequestEnvelope {
                    r#type: "create_session".into(),
                    version: 1,
                    request_id: id,
                    reply_to: None,
                    payload,
                },
                Duration::from_secs(30),
            );
            (r.await, bus.response_stream())
        })
    };
    assert!(h.program.wait_exit(Duration::from_secs(10)).is_some(), "handler didn't abort");
    let mut probe = h.probe().await;
    h.restart().await;
    let (response, stream) = waiting.await.unwrap();
    let response = response.expect("stored response");
    assert_eq!(response.status, Status::Ok, "{:?}", response.error);
    assert_eq!(h.count("SELECT count(*) FROM sessions"), 1);
    assert_eq!(h.device(ids.device_id).await["shot_count"], 5);
    let mut c = h.redis.connection().await;
    let entries: redis::streams::StreamRangeReply =
        redis::cmd("XRANGE").arg(&stream).arg("-").arg("+").query_async(&mut c).await.unwrap();
    let ours = entries
        .ids
        .iter()
        .filter(|e| redis::from_redis_value::<String>(e.map["data"].clone()).unwrap().contains(&id.to_string()))
        .count();
    assert_eq!(ours, 1);
    assert_eq!(changes(&mut probe, 1).await, ["all/null/reload"]);
}

// DB-IDM-04
#[tokio::test]
async fn db_idm_04_duplicate_after_the_image_moved() {
    let h = Handler::start().await;
    let ids = h.refs().await;
    let staged = h.stage("paper.jpg");
    let (first, second, after_first, after_second) = send_twice(&h, "create_session", cs_ref(ids, Some(&staged))).await;
    assert_eq!(second.status, Status::Ok, "{:?}", second.error);
    assert_eq!(second.payload, first.payload);
    assert_eq!(after_second, after_first);
}
