// doc/testing/shared_libraries.md#idempotency-store-integration-rust

use serde_json::json;
use targetscope_common::bus::Bus;
use targetscope_common::idempotency::RedisIdempotencyStore;
use targetscope_testkit::RedisTestServer;
use uuid::Uuid;

async fn store() -> (RedisTestServer, Bus, RedisIdempotencyStore) {
    let redis = RedisTestServer::start().await;
    let bus = Bus::connect(&redis.url(), "prog").await.unwrap();
    let s = RedisIdempotencyStore::new(&bus);
    (redis, bus, s)
}

// LIB-IDS-01
#[tokio::test(flavor = "multi_thread")]
async fn lib_ids_01_unknown_id() {
    let (_r, _b, s) = store().await;
    assert_eq!(s.get(&Uuid::new_v4()).await.unwrap(), None);
}

// LIB-IDS-02
#[tokio::test(flavor = "multi_thread")]
async fn lib_ids_02_recorded_in_a_transaction() {
    let (r, b, s) = store().await;
    let id = Uuid::new_v4();
    let mut pipe = redis::pipe();
    pipe.atomic().cmd("SET").arg("x").arg(1).ignore();
    s.record(&mut pipe, &id, &json!({ "ok": true }));
    let () = pipe.query_async(&mut b.connection()).await.unwrap();
    assert_eq!(s.get(&id).await.unwrap(), Some(json!({ "ok": true })));
    let mut c = r.connection().await;
    let ttl: i64 = redis::cmd("TTL").arg(format!("prog:processed:{id}")).query_async(&mut c).await.unwrap();
    assert!((86_390..=86_400).contains(&ttl), "{ttl}");
    let x: i64 = redis::cmd("GET").arg("x").query_async(&mut c).await.unwrap();
    assert_eq!(x, 1);
}

// LIB-IDS-03
#[tokio::test(flavor = "multi_thread")]
async fn lib_ids_03_discarded_transaction() {
    let (r, _b, s) = store().await;
    let id = Uuid::new_v4();
    let mut c = r.connection().await;
    let () = redis::cmd("MULTI").query_async(&mut c).await.unwrap();
    let mut pipe = redis::pipe();
    s.record(&mut pipe, &id, &json!({}));
    let _: redis::Value = pipe.query_async(&mut c).await.unwrap_or(redis::Value::Nil);
    let () = redis::cmd("DISCARD").query_async(&mut c).await.unwrap();
    assert_eq!(s.get(&id).await.unwrap(), None);
}
