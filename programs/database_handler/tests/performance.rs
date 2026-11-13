// doc/testing/database_handler.md#performance-performance

mod common;

use std::time::{Duration, Instant};

use common::*;
use serde_json::json;
use targetscope_testkit::fixtures::ReferenceIds;

const SESSIONS: i64 = 10_000;
const SHOTS: i64 = 60;

async fn large_database() -> (Handler, ReferenceIds) {
    let h = Handler::start().await;
    let ids = h.refs().await;
    let mut db = h.db();
    let tx = db.transaction().unwrap();
    {
        let mut session = tx
            .prepare(
                "INSERT INTO sessions (id, uuid, saved_at, score, grouping_score, distance_m, target_type_id, shooter_id,
                   device_id, single_handed, supported, shot_count, duration_s, notes, paper_count)
                 VALUES (?1, ?2, ?3, 500, 520, 10, ?4, ?5, ?6, 1, 0, ?7, 1200, 'perf', 1)",
            )
            .unwrap();
        let mut shot =
            tx.prepare("INSERT INTO shots (session_id, shot_no, paper_no, angle_turns, radius_mm) VALUES (?1, ?2, 1, ?3, ?4)").unwrap();
        for s in 1..=SESSIONS {
            session
                .execute(rusqlite::params![
                    s,
                    uuid::Uuid::new_v4().to_string(),
                    1_700_000_000 + s * 3600,
                    ids.target_type_id,
                    ids.shooter_id,
                    ids.device_id,
                    SHOTS
                ])
                .unwrap();
            for n in 1..=SHOTS {
                shot.execute(rusqlite::params![s, n, (n as f64) / (SHOTS as f64 + 1.0), (s % 90) as f64 + 0.5]).unwrap();
            }
        }
    }
    tx.commit().unwrap();
    (h, ids)
}

async fn timed(h: &Handler, r#type: &str, payload: serde_json::Value) -> (serde_json::Value, Duration) {
    let started = Instant::now();
    let r = h.ok(r#type, payload).await;
    (r, started.elapsed())
}

// DB-PRF-01
#[tokio::test(flavor = "multi_thread")]
#[ignore = "performance"]
async fn perf_db_prf_01_history_page() {
    let (h, _) = large_database().await;
    let (r, t) = timed(&h, "read_sessions", json!({ "filter": {}, "limit": 50, "include_shots": false })).await;
    assert_eq!(r["sessions"].as_array().unwrap().len(), 50);
    assert_eq!(r["total"], SESSIONS);
    assert!(t < Duration::from_millis(200), "{t:?}");
}

// DB-PRF-02
#[tokio::test(flavor = "multi_thread")]
#[ignore = "performance"]
async fn perf_db_prf_02_full_read() {
    let (h, _) = large_database().await;
    let mut read = 0;
    let mut slowest = Duration::ZERO;
    while read < SESSIONS {
        let payload = json!({
            "filter": {}, "order_by": { "field": "id", "direction": "asc" }, "limit": 500, "offset": read, "include_shots": true
        });
        let (r, t) = timed(&h, "read_sessions", payload).await;
        let page = r["sessions"].as_array().unwrap();
        assert!(page.iter().all(|s| s["shots"].as_array().unwrap().len() == SHOTS as usize));
        read += page.len() as i64;
        slowest = slowest.max(t);
    }
    assert_eq!(read, SESSIONS);
    assert!(slowest < Duration::from_secs(1), "{slowest:?}");
}

// DB-PRF-03
#[tokio::test(flavor = "multi_thread")]
#[ignore = "performance"]
async fn perf_db_prf_03_save() {
    let (h, ids) = large_database().await;
    let mut p = cs_ref(ids, None);
    let shots: Vec<_> =
        (0..SHOTS).map(|n| json!({ "angle_turns": n as f64 / 61.0, "radius_mm": 10.5, "paper_no": if n < 30 { 1 } else { 2 } })).collect();
    p["shots"] = json!(shots);
    p["shot_count"] = json!(SHOTS);
    p["paper_count"] = json!(2);
    p["images"] = json!([{ "paper_no": 1, "staged_file": h.stage("paper.jpg") }, { "paper_no": 2, "staged_file": h.stage("paper.jpg") }]);
    let (r, t) = timed(&h, "create_session", p).await;
    assert_eq!(r["images"].as_array().unwrap().len(), 2);
    assert!(t < Duration::from_millis(200), "{t:?}");
}
