// doc/testing/shared_libraries.md#redis-bus-integration

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use targetscope_common::bus::{Bus, BusError, ReadMode, RequestHandler};
use targetscope_common::messages::{ErrorCode, Request, ServiceError};
use targetscope_testkit::RedisTestServer;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
enum TestRequest {
    Echo(Echo),
    Fail(Fail),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Echo {
    value: i64,
    #[serde(default)]
    delay_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Fail {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct EchoResponse {
    value: i64,
}

impl Request for Echo {
    type Response = EchoResponse;
    const TYPE: &'static str = "echo";
    const STREAM: &'static str = "b:requests";
    const TIMEOUT: Duration = Duration::from_secs(5);
}

impl Request for Fail {
    type Response = Value;
    const TYPE: &'static str = "fail";
    const STREAM: &'static str = "b:requests";
    const TIMEOUT: Duration = Duration::from_secs(5);
}

#[derive(Debug, Clone, Serialize)]
struct Slow(Echo);

impl Request for Slow {
    type Response = EchoResponse;
    const TYPE: &'static str = "echo";
    const STREAM: &'static str = "b:requests";
    const TIMEOUT: Duration = Duration::from_millis(200);
}

#[derive(Default)]
struct Recorder {
    calls: AtomicUsize,
    ids: Mutex<Vec<Uuid>>,
}

struct Handler(Arc<Recorder>);

#[async_trait]
impl RequestHandler<TestRequest> for Handler {
    async fn handle(&self, id: Uuid, req: TestRequest) -> Result<Value, ServiceError> {
        self.0.calls.fetch_add(1, Ordering::SeqCst);
        self.0.ids.lock().unwrap().push(id);
        match req {
            TestRequest::Echo(e) => {
                tokio::time::sleep(Duration::from_millis(e.delay_ms)).await;
                Ok(json!({ "value": e.value }))
            }
            TestRequest::Fail(_) => Err(ServiceError::not_found("nothing here").with_details(json!({ "what": "x" }))),
        }
    }
}

struct Setup {
    redis: RedisTestServer,
    a: Bus,
    b: Bus,
    rec: Arc<Recorder>,
}

async fn setup(concurrent: bool) -> Setup {
    let redis = RedisTestServer::start().await;
    let a = Bus::connect(&redis.url(), "a").await.unwrap();
    let b = Bus::connect(&redis.url(), "b").await.unwrap();
    let rec = Arc::new(Recorder::default());
    let (serving, handler) = (b.clone(), Arc::new(Handler(rec.clone())));
    tokio::spawn(async move {
        if concurrent {
            serving.serve_concurrent(handler, 128).await
        } else {
            serving.serve(Handler(handler.0.clone())).await
        }
    });
    Setup { redis, a, b, rec }
}

async fn redis(s: &Setup) -> redis::aio::MultiplexedConnection {
    s.redis.connection().await
}

async fn pending(s: &Setup) -> i64 {
    let mut c = redis(s).await;
    let v: redis::Value = redis::cmd("XPENDING").arg("b:requests").arg("b").query_async(&mut c).await.unwrap();
    match v {
        redis::Value::Array(items) => match items.first() {
            Some(redis::Value::Int(n)) => *n,
            _ => 0,
        },
        _ => 0,
    }
}

async fn raw_request(s: &Setup, data: &str) {
    let mut c = redis(s).await;
    let _: String = redis::cmd("XADD").arg("b:requests").arg("*").arg("data").arg(data).query_async(&mut c).await.unwrap();
}

async fn responses(s: &Setup, stream: &str) -> Vec<Value> {
    let mut c = redis(s).await;
    let r: redis::streams::StreamRangeReply = redis::cmd("XRANGE").arg(stream).arg("-").arg("+").query_async(&mut c).await.unwrap();
    r.ids.iter().map(|e| serde_json::from_str(&redis::from_redis_value::<String>(e.map["data"].clone()).unwrap()).unwrap()).collect()
}

// LIB-BUS-01
#[tokio::test(flavor = "multi_thread")]
async fn lib_bus_01_request_and_response() {
    let s = setup(false).await;
    let r = s.a.ping("b", Duration::from_secs(5)).await.unwrap();
    assert_eq!(r.program, "b");
    assert_eq!(r.language, "rust");
    assert!(!r.program_version.is_empty());
    assert!(r.started_at <= std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64);
}

// LIB-BUS-02
#[tokio::test(flavor = "multi_thread")]
async fn lib_bus_02_responses_are_matched() {
    let s = setup(true).await;
    let mut tasks = Vec::new();
    for i in 0..100 {
        let a = s.a.clone();
        tasks.push(tokio::spawn(async move { a.request(Echo { value: i, delay_ms: (i as u64 * 37) % 50 }).await.unwrap() }));
    }
    for (i, t) in tasks.into_iter().enumerate() {
        assert_eq!(t.await.unwrap().value, i as i64);
    }
}

// LIB-BUS-03
#[tokio::test(flavor = "multi_thread")]
async fn lib_bus_03_timeout() {
    let s = setup(true).await;
    let start = Instant::now();
    let r = s.a.request(Slow(Echo { value: 1, delay_ms: 1000 })).await;
    let took = start.elapsed();
    assert!(matches!(r, Err(BusError::Timeout)));
    assert!(took >= Duration::from_millis(200) && took < Duration::from_millis(400), "{took:?}");
}

// LIB-BUS-04
#[tokio::test(flavor = "multi_thread")]
async fn lib_bus_04_late_response_is_dropped() {
    let s = setup(true).await;
    assert!(matches!(s.a.request(Slow(Echo { value: 1, delay_ms: 400 })).await, Err(BusError::Timeout)));
    tokio::time::sleep(Duration::from_millis(400)).await;
    assert_eq!(s.a.request(Echo { value: 2, delay_ms: 0 }).await.unwrap().value, 2);
}

// LIB-BUS-05
#[tokio::test(flavor = "multi_thread")]
async fn lib_bus_05_resend_with_the_same_id() {
    let s = setup(false).await;
    let id = Uuid::new_v4();
    s.a.request_with_id(id, Echo { value: 1, delay_ms: 0 }).await.unwrap();
    s.a.request_with_id(id, Echo { value: 1, delay_ms: 0 }).await.unwrap();
    assert_eq!(*s.rec.ids.lock().unwrap(), vec![id, id]);
}

// LIB-BUS-06
#[tokio::test(flavor = "multi_thread")]
async fn lib_bus_06_remote_error() {
    let s = setup(false).await;
    match s.a.request(Fail {}).await {
        Err(BusError::Remote(e)) => {
            assert_eq!(e.code, ErrorCode::NotFound);
            assert_eq!(e.message, "nothing here");
            assert_eq!(e.details, Some(json!({ "what": "x" })));
        }
        other => panic!("{other:?}"),
    }
}

// LIB-BUS-07
#[tokio::test(flavor = "multi_thread")]
async fn lib_bus_07_stream_entry_format() {
    let s = setup(false).await;
    s.a.request(Echo { value: 1, delay_ms: 0 }).await.unwrap();
    let mut c = redis(&s).await;
    let r: redis::streams::StreamRangeReply = redis::cmd("XRANGE").arg("b:requests").arg("-").arg("+").query_async(&mut c).await.unwrap();
    let entry = &r.ids[0];
    assert_eq!(entry.map.keys().collect::<Vec<_>>(), vec!["data"]);
    let v: Value = serde_json::from_str(&redis::from_redis_value::<String>(entry.map["data"].clone()).unwrap()).unwrap();
    assert_eq!(v["type"], "echo");
}

// LIB-BUS-08
#[tokio::test(flavor = "multi_thread")]
async fn lib_bus_08_unknown_request_type() {
    let s = setup(false).await;
    let id = Uuid::new_v4();
    raw_request(&s, &json!({ "type": "nonsense", "version": 1, "request_id": id, "reply_to": "x:responses", "payload": {} }).to_string())
        .await;
    let mut got = Vec::new();
    for _ in 0..100 {
        got = responses(&s, "x:responses").await;
        if !got.is_empty() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert_eq!(got[0]["error"]["code"], "invalid_message");
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(pending(&s).await, 0);
}

// LIB-BUS-09
#[tokio::test(flavor = "multi_thread")]
async fn lib_bus_09_unparseable_entry() {
    let s = setup(false).await;
    raw_request(&s, "garbage").await;
    s.a.request(Echo { value: 1, delay_ms: 0 }).await.unwrap();
    assert_eq!(pending(&s).await, 0);
    assert_eq!(s.rec.calls.load(Ordering::SeqCst), 1);
}

// LIB-BUS-10
#[tokio::test(flavor = "multi_thread")]
async fn lib_bus_10_unknown_version() {
    let s = setup(false).await;
    raw_request(
        &s,
        &json!({ "type": "echo", "version": 99, "request_id": Uuid::new_v4(), "reply_to": "y:responses", "payload": { "value": 1 } })
            .to_string(),
    )
    .await;
    s.a.request(Echo { value: 1, delay_ms: 0 }).await.unwrap();
    assert_eq!(responses(&s, "y:responses").await[0]["error"]["code"], "invalid_message");
    assert_eq!(pending(&s).await, 0);
}

// LIB-BUS-11
#[tokio::test(flavor = "multi_thread")]
async fn lib_bus_11_no_reply_to() {
    let s = setup(false).await;
    raw_request(&s, &json!({ "type": "echo", "version": 1, "request_id": Uuid::new_v4(), "payload": { "value": 1 } }).to_string()).await;
    s.a.request(Echo { value: 2, delay_ms: 0 }).await.unwrap();
    assert_eq!(s.rec.calls.load(Ordering::SeqCst), 2);
    assert_eq!(pending(&s).await, 0);
}

// LIB-BUS-13
#[tokio::test(flavor = "multi_thread")]
async fn lib_bus_13_consumer_group_already_exists() {
    let s = setup(false).await;
    s.a.request(Echo { value: 1, delay_ms: 0 }).await.unwrap();
    s.b.stop();
    let b2 = Bus::connect(&s.redis.url(), "b").await.unwrap();
    let rec = Arc::new(Recorder::default());
    let serving = b2.clone();
    let h = Handler(rec.clone());
    tokio::spawn(async move { serving.serve(h).await });
    assert_eq!(s.a.request(Echo { value: 3, delay_ms: 0 }).await.unwrap().value, 3);
}

// LIB-BUS-14
#[tokio::test(flavor = "multi_thread")]
async fn lib_bus_14_consumer_name() {
    let s = setup(false).await;
    s.a.request(Echo { value: 1, delay_ms: 0 }).await.unwrap();
    let mut c = redis(&s).await;
    let v: redis::Value = redis::cmd("XINFO").arg("CONSUMERS").arg("b:requests").arg("b").query_async(&mut c).await.unwrap();
    let text = format!("{v:?}");
    let host = hostname::get().unwrap().to_string_lossy().into_owned();
    assert!(text.contains(&format!("{host}-{}", std::process::id())), "{text}");
}

// LIB-BUS-15
#[tokio::test(flavor = "multi_thread")]
async fn lib_bus_15_stream_caps() {
    let s = setup(false).await;
    for i in 0..25_000 {
        s.b.publish(&json!({ "type": "tick", "payload": { "i": i } })).await.unwrap();
    }
    let mut c = redis(&s).await;
    let len: i64 = redis::cmd("XLEN").arg("b:events").query_async(&mut c).await.unwrap();
    assert!((10_000..=11_000).contains(&len), "{len}");
}

// LIB-BUS-16
#[tokio::test(flavor = "multi_thread")]
async fn lib_bus_16_published_event() {
    let s = setup(false).await;
    let env = s.b.publish(&json!({ "type": "shot_detected", "payload": { "hole_id": 1 } })).await.unwrap();
    assert_eq!(env.event_id.get_version_num(), 4);
    assert_eq!(env.source, "b");
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64;
    assert!((now - env.timestamp_ms).abs() < 1000);
    let mut c = redis(&s).await;
    let r: redis::streams::StreamRangeReply = redis::cmd("XRANGE").arg("b:events").arg("-").arg("+").query_async(&mut c).await.unwrap();
    let v: Value = serde_json::from_str(&redis::from_redis_value::<String>(r.ids[0].map["data"].clone()).unwrap()).unwrap();
    assert_eq!(v["type"], "shot_detected");
    assert_eq!(v["payload"]["hole_id"], 1);
}

// LIB-BUS-17
#[tokio::test(flavor = "multi_thread")]
async fn lib_bus_17_consumer_group_reader_keeps_its_position() {
    let s = setup(false).await;
    {
        let mut reader = s.a.events::<Value>("b", ReadMode::ConsumerGroup);
        let waiting = tokio::spawn(async move {
            for _ in 0..3 {
                let e = reader.next().await.unwrap();
                reader.ack(&e.entry_id).await.unwrap();
            }
        });
        tokio::time::sleep(Duration::from_millis(300)).await;
        for i in 0..3 {
            s.b.publish(&json!({ "type": "n", "payload": { "i": i } })).await.unwrap();
        }
        waiting.await.unwrap();
    }
    for i in 3..8 {
        s.b.publish(&json!({ "type": "n", "payload": { "i": i } })).await.unwrap();
    }
    let mut reader = s.a.events::<Value>("b", ReadMode::ConsumerGroup);
    for i in 3..8 {
        let e = reader.next().await.unwrap();
        assert_eq!(e.envelope.payload["i"], i);
        reader.ack(&e.entry_id).await.unwrap();
    }
}

// LIB-BUS-18
#[tokio::test(flavor = "multi_thread")]
async fn lib_bus_18_plain_reader_misses_nothing_at_startup() {
    let s = setup(false).await;
    let after = s.a.newest_event_id("b").await.unwrap();
    for i in 0..3 {
        s.b.publish(&json!({ "type": "n", "payload": { "i": i } })).await.unwrap();
    }
    let mut reader = s.a.events::<Value>("b", ReadMode::After(after));
    for i in 0..3 {
        assert_eq!(reader.next().await.unwrap().envelope.payload["i"], i);
    }
}

// LIB-BUS-19
#[tokio::test(flavor = "multi_thread")]
async fn lib_bus_19_plain_reader_continuity() {
    let s = setup(false).await;
    let after = s.a.newest_event_id("b").await.unwrap();
    let mut reader = s.a.events::<Value>("b", ReadMode::After(after));
    let b = s.b.clone();
    tokio::spawn(async move {
        for burst in 0..10 {
            for i in 0..100 {
                b.publish(&json!({ "type": "n", "payload": { "i": burst * 100 + i } })).await.unwrap();
            }
            tokio::time::sleep(Duration::from_millis(30)).await;
        }
    });
    for i in 0..1000 {
        assert_eq!(reader.next().await.unwrap().envelope.payload["i"], i);
    }
}

// LIB-BUS-20
#[tokio::test(flavor = "multi_thread")]
async fn lib_bus_20_redis_restart() {
    let mut s = setup(false).await;
    s.a.request(Echo { value: 1, delay_ms: 0 }).await.unwrap();
    s.redis.stop().await;
    assert!(s.a.request(Echo { value: 2, delay_ms: 0 }).await.is_err());
    tokio::time::sleep(Duration::from_secs(3)).await;
    s.redis.restart().await;
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        match s.a.request(Echo { value: 3, delay_ms: 0 }).await {
            Ok(r) => {
                assert_eq!(r.value, 3);
                break;
            }
            Err(e) => {
                assert!(Instant::now() < deadline, "no recovery: {e}");
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        }
    }
}

// LIB-BUS-21
#[tokio::test(flavor = "multi_thread")]
async fn lib_bus_21_ping_is_answered_by_the_serve_loop() {
    let s = setup(false).await;
    s.a.ping("b", Duration::from_secs(5)).await.unwrap();
    assert_eq!(s.rec.calls.load(Ordering::SeqCst), 0);
}

// LIB-BUS-22
#[tokio::test(flavor = "multi_thread")]
async fn lib_bus_22_serve_concurrent_answers_at_once() {
    let s = setup(true).await;
    s.a.request(Echo { value: 0, delay_ms: 0 }).await.unwrap();
    for i in 1..=5 {
        tokio::time::sleep(Duration::from_millis(1500)).await;
        let started = Instant::now();
        assert_eq!(s.a.request(Echo { value: i, delay_ms: 0 }).await.unwrap().value, i);
        assert!(started.elapsed() < Duration::from_millis(200), "request {i}: {:?}", started.elapsed());
    }
}

// LIB-BUS-23
#[tokio::test(flavor = "multi_thread")]
async fn lib_bus_23_many_pending_requests_at_start() {
    let redis = RedisTestServer::start().await;
    let a = Bus::connect(&redis.url(), "a").await.unwrap();
    let mut c = redis.connection().await;
    let _: () = redis::cmd("XGROUP").arg("CREATE").arg("b:requests").arg("b").arg("0").arg("MKSTREAM").query_async(&mut c).await.unwrap();
    let mut calls = Vec::new();
    for i in 0..250 {
        let a = a.clone();
        calls.push(tokio::spawn(async move { a.request(Echo { value: i, delay_ms: 0 }).await }));
    }
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut delivered = 0;
    while delivered < 250 {
        assert!(Instant::now() < deadline);
        let r: redis::streams::StreamReadReply = redis::cmd("XREADGROUP")
            .arg("GROUP")
            .arg("b")
            .arg("crashed")
            .arg("COUNT")
            .arg(1000)
            .arg("STREAMS")
            .arg("b:requests")
            .arg(">")
            .query_async(&mut c)
            .await
            .unwrap();
        delivered += r.keys.iter().map(|k| k.ids.len()).sum::<usize>();
    }
    let b = Bus::connect(&redis.url(), "b").await.unwrap();
    let rec = Arc::new(Recorder::default());
    let handler = Arc::new(Handler(rec.clone()));
    tokio::spawn(async move { b.serve_concurrent(handler, 16).await });
    for (i, call) in calls.into_iter().enumerate() {
        assert_eq!(call.await.unwrap().unwrap().value, i as i64);
    }
    assert_eq!(rec.calls.load(Ordering::SeqCst), 250);
    let v: redis::Value = redis::cmd("XPENDING").arg("b:requests").arg("b").query_async(&mut c).await.unwrap();
    assert!(matches!(v, redis::Value::Array(ref items) if items.first() == Some(&redis::Value::Int(0))), "{v:?}");
}
