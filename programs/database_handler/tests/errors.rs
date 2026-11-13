// doc/testing/database_handler.md#error-handling-integration

mod common;

use std::time::{Duration, Instant};

use common::*;
use serde_json::json;
use targetscope_common::messages::{ResponseEnvelope, Status};
use targetscope_testkit::TestBus;
use uuid::Uuid;

fn spawn_request(url: String, r#type: &'static str, payload: serde_json::Value) -> tokio::task::JoinHandle<Option<ResponseEnvelope>> {
    tokio::spawn(async move {
        let bus = TestBus::connect(&url).await;
        bus.try_send(P, Uuid::new_v4(), r#type, payload, Duration::from_secs(60)).await
    })
}

async fn reload_events(h: &Handler) -> usize {
    let mut c = h.redis.connection().await;
    let r: redis::streams::StreamRangeReply =
        redis::cmd("XRANGE").arg("database_handler:events").arg("-").arg("+").query_async(&mut c).await.unwrap();
    r.ids.iter().filter(|e| redis::from_redis_value::<String>(e.map["data"].clone()).unwrap().contains("\"reload\"")).count()
}

// DB-ERR-01
#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "failpoints"), ignore = "needs --features failpoints")]
async fn db_err_01_redis_lost() {
    let mut h = Handler::start().await;
    let ids = h.refs().await;
    h.restart_with(&[("FAILPOINTS", "db.before_commit=sleep(1500)")]).await;
    let reloads_before = reload_events(&h).await;
    let pending: Vec<_> = (0..3).map(|_| spawn_request(h.redis.url(), "create_session", cs_ref(ids, None))).collect();
    tokio::time::sleep(Duration::from_millis(300)).await;
    h.redis.stop().await;
    tokio::time::sleep(Duration::from_secs(2)).await;
    h.redis.restart().await;
    for p in pending {
        let r = p.await.unwrap().expect("response after the outage");
        assert_eq!(r.status, Status::Ok, "{:?}", r.error);
    }
    assert_eq!(h.count("SELECT count(*) FROM sessions"), 3);
    let deadline = Instant::now() + Duration::from_secs(10);
    while reload_events(&h).await <= reloads_before {
        assert!(Instant::now() < deadline, "no reload event after reconnecting");
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

// DB-ERR-02
#[tokio::test]
async fn db_err_02_disk_full() {
    let h = Handler::start().await;
    let Some(small) = h.dir.small_tmpfs(1024 * 1024) else {
        eprintln!("mounting a tmpfs needs root; DB-ERR-02 not checked");
        return;
    };
    let ids = h.refs().await;
    h.ok("change_image_dir", json!({ "path": small.display().to_string(), "move_images": false })).await;
    let big = vec![0xffu8, 0xd8, 0xff].into_iter().chain(std::iter::repeat_n(0u8, 2 * 1024 * 1024)).collect::<Vec<u8>>();
    let staged = targetscope_testkit::fixtures::stage_bytes(&h.dir.staging_dir, "upload_big.jpg", &big);
    assert_eq!(h.err("create_session", cs_ref(ids, Some(&staged))).await, "io_error");
    assert_eq!(h.count("SELECT count(*) FROM sessions"), 0);
    assert_eq!(std::fs::read(h.dir.staging_dir.join(&staged)).unwrap(), big);
}

// DB-ERR-03
#[tokio::test(flavor = "multi_thread")]
async fn db_err_03_database_locked() {
    let h = Handler::start().await;
    let ids = h.refs().await;
    let path = h.dir.db_path();
    let (locked_tx, locked_rx) = std::sync::mpsc::channel();
    let holder = std::thread::spawn(move || {
        let c = rusqlite::Connection::open(path).unwrap();
        c.execute_batch("BEGIN EXCLUSIVE").unwrap();
        locked_tx.send(()).unwrap();
        std::thread::sleep(Duration::from_secs(6));
        c.execute_batch("COMMIT").unwrap();
    });
    locked_rx.recv().unwrap();
    let started = Instant::now();
    let r = h.bus.try_send(P, Uuid::new_v4(), "create_session", cs_ref(ids, None), Duration::from_secs(20)).await.unwrap();
    let waited = started.elapsed();
    assert_eq!(r.error.unwrap().code.as_str(), "database_error");
    assert!(waited >= Duration::from_millis(4500) && waited < Duration::from_millis(6500), "{waited:?}");
    holder.join().unwrap();
    h.ok("create_session", cs_ref(ids, None)).await;
    let mut c = h.redis.connection().await;
    let v: redis::Value = redis::cmd("XPENDING").arg("database_handler:requests").arg(P).query_async(&mut c).await.unwrap();
    assert!(matches!(v, redis::Value::Array(ref items) if items.first() == Some(&redis::Value::Int(0))), "{v:?}");
}

// DB-ERR-04
#[tokio::test(flavor = "multi_thread")]
async fn db_err_04_many_writes_at_once() {
    let h = Handler::start().await;
    let ids = h.refs().await;
    let pending: Vec<_> = (0..50).map(|_| spawn_request(h.redis.url(), "create_session", cs_ref(ids, None))).collect();
    for p in pending {
        let r = p.await.unwrap().expect("response");
        assert_eq!(r.status, Status::Ok, "{:?}", r.error);
    }
    assert_eq!(h.count("SELECT count(*) FROM sessions"), 50);
    assert_eq!(h.device(ids.device_id).await["shot_count"], 250);
}

// DB-ERR-05
#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "failpoints"), ignore = "needs --features failpoints")]
async fn db_err_05_shutdown_during_a_request() {
    let mut h = Handler::start().await;
    let ids = h.refs().await;
    h.restart_with(&[("FAILPOINTS", "db.before_commit=sleep(2000)")]).await;
    let pending = spawn_request(h.redis.url(), "create_session", cs_ref(ids, None));
    tokio::time::sleep(Duration::from_millis(500)).await;
    let status = tokio::task::block_in_place(|| h.program.terminate(Duration::from_secs(10))).expect("exit status");
    let r = pending.await.unwrap().expect("answered before exit");
    assert_eq!(r.status, Status::Ok, "{:?}", r.error);
    assert!(status.success(), "{status:?}");
    assert_eq!(h.count("SELECT count(*) FROM sessions"), 1);
}
