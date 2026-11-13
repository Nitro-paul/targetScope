// doc/testing/database_handler.md#setup
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::{json, Value};
use targetscope_common::messages::{ResponseEnvelope, Status};
use targetscope_testkit::fixtures::{self, ReferenceIds};
use targetscope_testkit::{EventProbe, ProgramUnderTest, RedisTestServer, TempDataDir, TestBus};
use uuid::Uuid;

pub const P: &str = "database_handler";
pub const EVENT_WAIT: Duration = Duration::from_secs(3);
pub const NO_EVENT_WAIT: Duration = Duration::from_millis(500);

pub fn binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_database_handler"))
}

pub struct Handler {
    pub redis: RedisTestServer,
    pub dir: TempDataDir,
    pub program: ProgramUnderTest,
    pub bus: TestBus,
    env: Vec<(String, String)>,
}

impl Handler {
    pub async fn start() -> Self {
        Self::start_with(&[]).await
    }

    pub async fn start_with(extra_env: &[(&str, &str)]) -> Self {
        let redis = RedisTestServer::start().await;
        let dir = TempDataDir::new();
        Self::start_in(redis, dir, extra_env).await
    }

    pub async fn start_in(redis: RedisTestServer, dir: TempDataDir, extra_env: &[(&str, &str)]) -> Self {
        let mut env = dir.env(&redis.url());
        env.extend(extra_env.iter().map(|(k, v)| (k.to_string(), v.to_string())));
        let program = ProgramUnderTest::start(&binary(), P, &env).await;
        let bus = TestBus::connect(&redis.url()).await;
        Handler { redis, dir, program, bus, env }
    }

    pub async fn restart(&mut self) {
        self.restart_with(&[]).await;
    }

    pub async fn restart_with(&mut self, extra_env: &[(&str, &str)]) {
        self.program.terminate(Duration::from_secs(10));
        let mut env = self.dir.env(&self.redis.url());
        env.extend(extra_env.iter().map(|(k, v)| (k.to_string(), v.to_string())));
        self.env = env;
        self.program = ProgramUnderTest::start(&binary(), P, &self.env).await;
    }

    pub async fn send(&self, r#type: &str, payload: Value) -> ResponseEnvelope {
        self.bus.send(P, r#type, payload).await
    }

    pub async fn ok(&self, r#type: &str, payload: Value) -> Value {
        let r = self.send(r#type, payload).await;
        assert_eq!(r.status, Status::Ok, "{type}: {:?}", r.error);
        r.payload.unwrap()
    }

    pub async fn err(&self, r#type: &str, payload: Value) -> String {
        let r = self.send(r#type, payload).await;
        assert_eq!(r.status, Status::Error, "{type} succeeded: {:?}", r.payload);
        r.error.unwrap().code.as_str().to_string()
    }

    pub async fn probe(&self) -> EventProbe {
        EventProbe::start(&self.redis.url(), P).await
    }

    pub async fn refs(&self) -> ReferenceIds {
        fixtures::create_reference_records(&self.bus).await
    }

    pub fn stage(&self, image: &str) -> String {
        fixtures::stage_image(&self.dir.staging_dir, image)
    }

    pub fn db(&self) -> rusqlite::Connection {
        rusqlite::Connection::open(self.dir.db_path()).unwrap()
    }

    pub fn count(&self, sql: &str) -> i64 {
        self.db().query_row(sql, [], |r| r.get(0)).unwrap()
    }

    pub fn image(&self, image_path: &str) -> PathBuf {
        self.dir.image_dir.join(image_path)
    }

    pub async fn setting(&self, name: &str) -> Value {
        let r = self
            .ok("read_settings", json!({ "filter": { "conditions": [{ "field": "name", "op": "eq", "value": name }] } }))
            .await;
        r["settings"][0].clone()
    }

    pub async fn device(&self, id: i64) -> Value {
        let r = self.ok("read_devices", json!({ "filter": { "conditions": [{ "field": "id", "op": "eq", "value": id }] } })).await;
        r["devices"][0].clone()
    }

    pub async fn session(&self, id: i64) -> Value {
        let r = self.ok("read_sessions", json!({ "filter": { "session_ids": [id] } })).await;
        r["sessions"][0].clone()
    }

    pub fn cs_ref(&self, ids: ReferenceIds, staged: Option<&str>) -> Value {
        cs_ref(ids, staged)
    }

    pub async fn create_session(&self, ids: ReferenceIds) -> Value {
        let staged = self.stage("paper.jpg");
        self.ok("create_session", self.cs_ref(ids, Some(&staged))).await
    }
}

// doc/testing/database_handler.md#setup
pub fn cs_ref(ids: ReferenceIds, staged: Option<&str>) -> Value {
    let images = match staged {
        Some(s) => json!([{ "paper_no": 1, "staged_file": s }]),
        None => json!([]),
    };
    json!({
        "shooter_id": ids.shooter_id, "device_id": ids.device_id, "target_type_id": ids.target_type_id,
        "distance_m": 10, "single_handed": true, "supported": false,
        "shots": fixtures::s_ref_shots(), "shot_count": 5, "paper_count": 1,
        "score": 38, "grouping_score": 45, "duration_s": 600, "notes": "test", "images": images
    })
}

pub fn with(mut v: Value, field: &str, value: Value) -> Value {
    v[field] = value;
    v
}

pub fn without(mut v: Value, field: &str) -> Value {
    v.as_object_mut().unwrap().remove(field);
    v
}

pub fn files_in(dir: &Path) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(dir)
        .map(|r| r.flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect())
        .unwrap_or_default();
    v.sort();
    v
}

pub fn is_uuid_v4(s: &str) -> bool {
    Uuid::parse_str(s).is_ok_and(|u| u.get_version_num() == 4)
}

pub fn now_s() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64
}

pub async fn changes(probe: &mut EventProbe, n: usize) -> Vec<String> {
    let mut out = Vec::new();
    for _ in 0..n {
        let e = probe.expect("data_changed", EVENT_WAIT).await;
        let key = match &e.payload["key"] {
            Value::String(s) => s.clone(),
            other => other.to_string(),
        };
        out.push(format!("{}/{}/{}", e.payload["table"].as_str().unwrap(), key, e.payload["action"].as_str().unwrap()));
    }
    out
}
