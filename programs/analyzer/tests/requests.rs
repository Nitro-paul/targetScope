// doc/testing/analyzer.md#requests-integration

mod common;

use std::path::Path;
use std::time::{Duration, Instant};

use analyzer::scoring::analyze;
use common::{s_ref, shot, tt_ref, Rng, DEV_REF_DIAMETER};
use serde_json::{json, Value};
use targetscope_common::messages::{ResponseEnvelope, Status};
use targetscope_common::model::{Shot, TargetType};
use targetscope_testkit::{ProgramUnderTest, RedisTestServer, TempDataDir, TestBus};
use uuid::Uuid;

const P: &str = "analyzer";

struct Analyzer {
    redis: RedisTestServer,
    _dir: TempDataDir,
    _program: ProgramUnderTest,
    bus: TestBus,
}

impl Analyzer {
    async fn start() -> Self {
        let redis = RedisTestServer::start().await;
        let dir = TempDataDir::new();
        let program = ProgramUnderTest::start(Path::new(env!("CARGO_BIN_EXE_analyzer")), P, &dir.env(&redis.url())).await;
        let bus = TestBus::connect(&redis.url()).await;
        Analyzer { redis, _dir: dir, _program: program, bus }
    }

    async fn send(&self, payload: Value) -> ResponseEnvelope {
        self.bus.try_send(P, Uuid::new_v4(), "analyze_shots", payload, Duration::from_secs(60)).await.expect("response")
    }
}

fn payload(shots: &[Shot], t: &TargetType, bullet_diameter_mm: f64) -> Value {
    json!({ "shots": shots, "target_type": t, "bullet_diameter_mm": bullet_diameter_mm })
}

fn error_code(r: &ResponseEnvelope) -> String {
    r.error.as_ref().map(|e| e.code.as_str().to_string()).unwrap_or_else(|| format!("ok: {:?}", r.payload))
}

// AN-REQ-01
#[tokio::test]
async fn an_req_01_valid_request() {
    let a = Analyzer::start().await;
    let r = a.send(payload(&s_ref(), &tt_ref(), DEV_REF_DIAMETER)).await;
    assert_eq!(r.status, Status::Ok, "{:?}", r.error);
    let expected = serde_json::to_value(analyze(&s_ref(), &tt_ref(), DEV_REF_DIAMETER)).unwrap();
    assert_eq!(r.payload.unwrap(), expected);
    assert_eq!(expected["shot_scores"], json!([12, 11, 9, 6, 0]));
    assert_eq!(expected["score"], 38);
}

// AN-REQ-02
#[tokio::test]
async fn an_req_02_invalid_bullet_diameter() {
    let a = Analyzer::start().await;
    assert_eq!(error_code(&a.send(payload(&s_ref(), &tt_ref(), 0.0)).await), "validation_failed");
}

// AN-REQ-03
#[tokio::test]
async fn an_req_03_invalid_target_type() {
    let a = Analyzer::start().await;
    // doc/testing/shared_libraries.md#ring-generation-and-target-type-validation-unit
    let cases = [
        TargetType { inner_white_score: 7, ..tt_ref() },
        TargetType { black_score: 1, ..tt_ref() },
        TargetType { outer_score: 0, ..tt_ref() },
        TargetType { inner_white_radius_mm: 0.0, ..tt_ref() },
        TargetType { inner_white_radius_mm: 30.0, ..tt_ref() },
        TargetType { black_radius_mm: 90.0, ..tt_ref() },
        TargetType { center_x_mm: 89.0, ..tt_ref() },
        TargetType { center_x_mm: 111.0, ..tt_ref() },
        TargetType { center_y_mm: 89.0, ..tt_ref() },
        TargetType { center_y_mm: 111.0, ..tt_ref() },
        TargetType { paper_width_mm: 0.0, ..tt_ref() },
    ];
    for t in cases {
        assert_eq!(error_code(&a.send(payload(&s_ref(), &t, DEV_REF_DIAMETER)).await), "validation_failed", "{t:?}");
    }
}

// AN-REQ-04
#[tokio::test]
async fn an_req_04_invalid_shot() {
    let a = Analyzer::start().await;
    for bad in [json!({ "angle_turns": 1.0, "radius_mm": 5.0 }), json!({ "angle_turns": 0.5, "radius_mm": -1.0 })] {
        let mut p = payload(&s_ref(), &tt_ref(), DEV_REF_DIAMETER);
        p["shots"][1] = bad.clone();
        assert_eq!(error_code(&a.send(p).await), "validation_failed", "{bad}");
    }
}

// AN-REQ-05
#[tokio::test(flavor = "multi_thread")]
async fn an_req_05_several_requests_at_once() {
    let a = Analyzer::start().await;
    let sets: Vec<Vec<Shot>> = (0..8)
        .map(|i| {
            let mut rng = Rng(100 + i);
            (0..60).map(|_| shot(rng.next(), rng.next() * 90.0)).collect()
        })
        .collect();
    let url = a.redis.url();
    let calls: Vec<_> = sets
        .iter()
        .map(|shots| {
            let (url, p) = (url.clone(), payload(shots, &tt_ref(), DEV_REF_DIAMETER));
            tokio::spawn(async move {
                let bus = TestBus::connect(&url).await;
                bus.try_send(P, Uuid::new_v4(), "analyze_shots", p, Duration::from_secs(120)).await
            })
        })
        .collect();
    let mut slowest_ping = Duration::ZERO;
    while !calls.iter().all(|c| c.is_finished()) {
        let started = Instant::now();
        a.bus.bus.ping(P, Duration::from_secs(5)).await.expect("ping answered while analyzing");
        slowest_ping = slowest_ping.max(started.elapsed());
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert!(slowest_ping < Duration::from_secs(1), "{slowest_ping:?}");
    for (call, shots) in calls.into_iter().zip(&sets) {
        let r = call.await.unwrap().expect("response");
        assert_eq!(r.status, Status::Ok, "{:?}", r.error);
        assert_eq!(r.payload.unwrap(), serde_json::to_value(analyze(shots, &tt_ref(), DEV_REF_DIAMETER)).unwrap());
    }
}
