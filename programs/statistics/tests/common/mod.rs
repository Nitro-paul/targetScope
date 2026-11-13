// doc/testing/statistics.md#setup
#![allow(dead_code)]

use std::path::Path;
use std::time::Duration;

use serde_json::{json, Value};
use targetscope_common::messages::{ResponseEnvelope, Status};
use targetscope_testkit::{Answer, FakeProgram, ProgramUnderTest, RedisTestServer, Responder, TempDataDir, TestBus};
use uuid::Uuid;

pub const P: &str = "statistics";
pub const TT_REF: i64 = 1;
pub const TT_2: i64 = 2;

pub struct Stats {
    pub redis: RedisTestServer,
    pub dir: TempDataDir,
    pub db: FakeProgram,
    pub program: ProgramUnderTest,
    pub bus: TestBus,
}

impl Stats {
    pub async fn start(sessions: Vec<Value>) -> Self {
        let redis = RedisTestServer::start().await;
        let dir = TempDataDir::new();
        let db = FakeProgram::start(&redis.url(), "database_handler").await;
        serve(&db, sessions, devices());
        let program = start_statistics(&redis, &dir).await;
        let bus = TestBus::connect(&redis.url()).await;
        Stats { redis, dir, db, program, bus }
    }

    pub async fn send(&self, payload: Value) -> ResponseEnvelope {
        self.bus.try_send(P, Uuid::new_v4(), "compute_statistics", payload, Duration::from_secs(90)).await.expect("response")
    }

    pub async fn ok(&self, payload: Value) -> Value {
        let r = self.send(payload).await;
        assert_eq!(r.status, Status::Ok, "{:?}", r.error);
        r.payload.unwrap()
    }

    pub async fn groups(&self, group_by: &str) -> Vec<Value> {
        let r = self.ok(json!({ "filter": {}, "group_by": group_by, "period": "month" })).await;
        r["groups"].as_array().unwrap().clone()
    }
}

// doc/testing/statistics.md#setup
pub async fn start_statistics(redis: &RedisTestServer, dir: &TempDataDir) -> ProgramUnderTest {
    let mut env = dir.env(&redis.url());
    env.push(("TZ".into(), "Europe/Berlin".into()));
    ProgramUnderTest::start(Path::new(env!("CARGO_BIN_EXE_statistics")), P, &env).await
}

pub fn devices() -> Vec<Value> {
    vec![
        json!({ "id": 1, "name": "D1", "category": "air pistol", "caliber": ".177", "bullet_diameter_mm": 4.5, "description": null }),
        json!({ "id": 2, "name": "D2", "category": "air rifle", "caliber": ".177", "bullet_diameter_mm": 4.5, "description": null }),
        json!({ "id": 3, "name": "D3", "category": "pistol", "caliber": ".22 LR", "bullet_diameter_mm": 5.6, "description": null }),
    ]
}

pub fn serve(db: &FakeProgram, sessions: Vec<Value>, devices: Vec<Value>) {
    db.on(
        "read_sessions",
        Responder::answer(move |p| {
            let offset = p["offset"].as_u64().unwrap_or(0) as usize;
            let limit = p["limit"].as_u64().map(|l| l as usize).unwrap_or(usize::MAX);
            let page: Vec<Value> = sessions.iter().skip(offset).take(limit).cloned().collect();
            Answer::Ok(json!({ "sessions": page, "total": sessions.len() }))
        }),
    );
    let by_ids = |records: Vec<Value>, key: &'static str| {
        Responder::answer(move |p| {
            let ids = p["filter"]["conditions"][0]["value"].as_array().cloned().unwrap_or_default();
            let found: Vec<Value> = records.iter().filter(|r| ids.contains(&r["id"])).cloned().collect();
            Answer::Ok(json!({ key: found }))
        })
    };
    db.on("read_devices", by_ids(devices, "devices"));
    let target_types = vec![target_type(TT_REF, "TT-REF"), target_type(TT_2, "TT-2")];
    db.on("read_target_types", by_ids(target_types, "target_types"));
}

pub fn target_type(id: i64, name: &str) -> Value {
    json!({
        "id": id, "name": name, "paper_width_mm": 200.0, "paper_height_mm": 200.0, "center_x_mm": 100.0, "center_y_mm": 100.0,
        "inner_white_radius_mm": 5.0, "inner_white_score": 12, "black_radius_mm": 30.0, "black_score": 7,
        "outer_radius_mm": 90.0, "outer_score": 1
    })
}

pub struct S {
    pub id: i64,
    pub shooter_id: Option<i64>,
    pub device_id: i64,
    pub target_type_id: i64,
    pub distance_m: Option<i64>,
    pub single_handed: bool,
    pub supported: bool,
    pub saved_at: i64,
    pub shots: Vec<(f64, f64)>,
}

impl Default for S {
    fn default() -> Self {
        S {
            id: 1,
            shooter_id: Some(1),
            device_id: 1,
            target_type_id: TT_REF,
            distance_m: Some(10),
            single_handed: true,
            supported: false,
            saved_at: 1_767_000_000,
            shots: vec![(0.25, 5.5), (0.5, 7.5)],
        }
    }
}

impl S {
    pub fn json(&self) -> Value {
        let shots: Vec<Value> = self.shots.iter().map(|(a, r)| json!({ "angle_turns": a, "radius_mm": r, "paper_no": 1 })).collect();
        json!({
            "id": self.id, "uuid": Uuid::new_v4(), "saved_at": self.saved_at, "score": 10 * shots.len(),
            "grouping_score": 11 * shots.len(), "distance_m": self.distance_m, "target_type_id": self.target_type_id,
            "shooter_id": self.shooter_id, "device_id": self.device_id, "single_handed": self.single_handed,
            "supported": self.supported, "shot_count": shots.len(), "duration_s": null, "notes": null, "paper_count": 1,
            "images": [], "shots": shots
        })
    }
}

pub fn keys(groups: &[Value]) -> Vec<Value> {
    groups.iter().map(|g| g["group_key"].clone()).collect()
}
