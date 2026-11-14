// doc/testing/statistics.md#performance-performance

mod common;

use std::time::{Duration, Instant};

use common::*;
use serde_json::json;
use targetscope_testkit::{fixtures, ProgramUnderTest, RedisTestServer, TempDataDir, TestBus};
use uuid::Uuid;

const SESSIONS: i64 = 10_000;
const SHOTS: i64 = 60;
const SHOOTERS: i64 = 5;

// ST-PRF-01
#[tokio::test(flavor = "multi_thread")]
#[ignore = "performance"]
async fn perf_st_prf_01_large_database() {
    let redis = RedisTestServer::start().await;
    let db_dir = TempDataDir::new();
    let db_binary = targetscope_testkit::process::program_binary("database_handler");
    let _database_handler = ProgramUnderTest::start(&db_binary, "database_handler", &db_dir.env(&redis.url())).await;
    let bus = TestBus::connect(&redis.url()).await;
    let ids = fixtures::create_reference_records(&bus).await;
    let mut shooters = vec![ids.shooter_id];
    for n in 2..=SHOOTERS {
        let r = bus.send("database_handler", "save_shooter", json!({ "name": format!("Shooter {n}") })).await;
        shooters.push(r.payload.unwrap()["id"].as_i64().unwrap());
    }

    let mut db = rusqlite::Connection::open(db_dir.db_path()).unwrap();
    let tx = db.transaction().unwrap();
    {
        let mut session = tx
            .prepare(
                "INSERT INTO sessions (id, uuid, saved_at, score, grouping_score, distance_m, target_type_id, shooter_id,
                   device_id, single_handed, supported, shot_count, duration_s, notes, paper_count)
                 VALUES (?1, ?2, ?3, 500, 520, 10, ?4, ?5, ?6, 1, 0, ?7, 1200, NULL, 1)",
            )
            .unwrap();
        let mut shot =
            tx.prepare("INSERT INTO shots (session_id, shot_no, paper_no, angle_turns, radius_mm) VALUES (?1, ?2, 1, ?3, ?4)").unwrap();
        for s in 1..=SESSIONS {
            let shooter = shooters[(s % SHOOTERS) as usize];
            let saved_at = 1_700_000_000 + s * 3600;
            session
                .execute(rusqlite::params![s, Uuid::new_v4().to_string(), saved_at, ids.target_type_id, shooter, ids.device_id, SHOTS])
                .unwrap();
            for n in 1..=SHOTS {
                shot.execute(rusqlite::params![s, n, n as f64 / (SHOTS as f64 + 1.0), (s % 90) as f64 + 0.5]).unwrap();
            }
        }
    }
    tx.commit().unwrap();

    let stats_dir = TempDataDir::new();
    let _statistics = start_statistics(&redis, &stats_dir).await;
    let started = Instant::now();
    let r = bus
        .try_send(
            P,
            Uuid::new_v4(),
            "compute_statistics",
            json!({ "filter": {}, "group_by": "shooter", "period": "week" }),
            Duration::from_secs(90),
        )
        .await
        .expect("response");
    let elapsed = started.elapsed();
    let payload = r.payload.unwrap_or_else(|| panic!("{:?}", r.error));
    assert_eq!(payload["session_count"], SESSIONS);
    assert_eq!(payload["groups"].as_array().unwrap().len(), SHOOTERS as usize);
    assert!(elapsed < Duration::from_secs(10), "{elapsed:?}");
}
