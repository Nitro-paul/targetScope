// doc/testing/database_handler.md#startup-and-schema-creation-integration

mod common;

use std::time::Duration;

use common::*;
use database_handler::db::{schema, Database};
use serde_json::json;
use targetscope_testkit::{ProgramUnderTest, RedisTestServer, TempDataDir};

fn names(c: &rusqlite::Connection, kind: &str) -> Vec<String> {
    let mut stmt = c.prepare("SELECT name FROM sqlite_master WHERE type = ?1 AND name NOT LIKE 'sqlite_%' ORDER BY name").unwrap();
    stmt.query_map([kind], |r| r.get(0)).unwrap().map(Result::unwrap).collect()
}

fn user_version(c: &rusqlite::Connection) -> i64 {
    c.query_row("PRAGMA user_version", [], |r| r.get(0)).unwrap()
}

fn log_text(dir: &TempDataDir) -> String {
    files_in(&dir.log_dir).iter().map(|f| std::fs::read_to_string(dir.log_dir.join(f)).unwrap_or_default()).collect()
}

// DB-INI-01
#[tokio::test]
async fn db_ini_01_first_start() {
    let redis = RedisTestServer::start().await;
    let dir = TempDataDir::new();
    std::fs::remove_dir_all(&dir.data_dir).unwrap();
    let h = Handler::start_in(redis, dir, &[]).await;
    assert!(h.dir.db_path().is_file());
    let db = h.db();
    assert_eq!(user_version(&db), schema::MIGRATIONS.len() as i64);
    assert_eq!(
        names(&db, "table"),
        ["devices", "processed_requests", "session_images", "sessions", "settings", "shooters", "shots", "target_types"]
    );
    assert_eq!(names(&db, "index").len(), 6, "{:?}", names(&db, "index"));
    assert_eq!(names(&db, "trigger").len(), 6, "{:?}", names(&db, "trigger"));
    assert!(h.dir.image_dir.join("devices").is_dir());
    assert!(h.dir.image_dir.join("sessions").is_dir());
    assert!(h.dir.staging_dir.is_dir());
}

// DB-INI-02
#[tokio::test]
async fn db_ini_02_connection_settings() {
    let dir = TempDataDir::new();
    let db = Database::open(&dir.db_path(), &dir.data_dir).unwrap();
    let pragma = |name: &str| db.conn.query_row(&format!("PRAGMA {name}"), [], |r| r.get::<_, rusqlite::types::Value>(0)).unwrap();
    assert_eq!(pragma("foreign_keys"), rusqlite::types::Value::Integer(1));
    assert_eq!(pragma("journal_mode"), rusqlite::types::Value::Text("wal".into()));
    assert_eq!(pragma("busy_timeout"), rusqlite::types::Value::Integer(5000));
}

// DB-INI-03
#[tokio::test]
async fn db_ini_03_default_settings() {
    let h = Handler::start().await;
    let settings = h.ok("read_settings", json!({})).await["settings"].as_array().unwrap().clone();
    assert_eq!(settings.len(), 20);
    for s in &settings {
        assert_eq!(s["value"], s["default_value"], "{}", s["name"]);
        assert_eq!(s["active"], s["default_active"], "{}", s["name"]);
    }
    let get = |name: &str| settings.iter().find(|s| s["name"] == name).unwrap().clone();
    assert_eq!(get("image_dir")["value"], h.dir.data_dir.join("images").display().to_string());
    assert_eq!(get("staging_dir")["value"], h.dir.data_dir.join("staging").display().to_string());
    assert_eq!(get("export_dir")["active"], false);
    assert_eq!(get("camera_rtsp_url")["active"], false);
}

// DB-INI-04
#[tokio::test]
async fn db_ini_04_user_changes_kept() {
    let mut h = Handler::start().await;
    h.ok("save_setting", json!({ "name": "camera_fps", "display_name": "x", "value": "15", "active": true })).await;
    h.restart().await;
    assert_eq!(h.setting("camera_fps").await["value"], "15");
}

// DB-INI-05
#[tokio::test]
async fn db_ini_05_no_target_types() {
    let h = Handler::start().await;
    assert_eq!(h.ok("read_target_types", json!({})).await["target_types"], json!([]));
}

// DB-INI-06
#[tokio::test]
async fn db_ini_06_migration_of_an_older_database() {
    let dir = TempDataDir::new();
    let mut c = rusqlite::Connection::open(dir.db_path()).unwrap();
    schema::migrate(&mut c, schema::MIGRATIONS).unwrap();
    c.execute("INSERT INTO shooters (uuid, name) VALUES ('0b3f1c2e-6c1a-4a8e-9d55-2f6b1d6c7a10', 'Alex')", []).unwrap();
    let mut with_test = schema::MIGRATIONS.to_vec();
    with_test.push("CREATE TABLE test_migration (x INTEGER); INSERT INTO test_migration VALUES (2);");
    schema::migrate(&mut c, &with_test).unwrap();
    assert_eq!(user_version(&c), 2);
    assert_eq!(c.query_row("SELECT count(*) FROM test_migration", [], |r| r.get::<_, i64>(0)).unwrap(), 1);
    assert_eq!(c.query_row("SELECT name FROM shooters", [], |r| r.get::<_, String>(0)).unwrap(), "Alex");
}

// DB-INI-07
#[tokio::test]
async fn db_ini_07_failing_migration() {
    let dir = TempDataDir::new();
    let mut c = rusqlite::Connection::open(dir.db_path()).unwrap();
    schema::migrate(&mut c, schema::MIGRATIONS).unwrap();
    let mut with_test = schema::MIGRATIONS.to_vec();
    with_test.push("CREATE TABLE test_migration (x INTEGER); THIS IS NOT SQL;");
    assert!(schema::migrate(&mut c, &with_test).is_err());
    assert_eq!(user_version(&c), 1);
    assert!(!names(&c, "table").contains(&"test_migration".to_string()));
    drop(c);

    let redis = RedisTestServer::start().await;
    let broken = TempDataDir::new();
    std::fs::write(broken.db_path(), b"this is not a database file, it is a text file that is long enough").unwrap();
    let mut p = ProgramUnderTest::spawn(&binary(), P, &broken.env(&redis.url()));
    let status = p.wait_exit(Duration::from_secs(10)).expect("program exits");
    assert!(!status.success());
}

// DB-INI-08
#[tokio::test]
async fn db_ini_08_orphan_files() {
    let redis = RedisTestServer::start().await;
    let dir = TempDataDir::new();
    let orphan = dir.image_dir.join("sessions/orphan_1.jpg");
    std::fs::create_dir_all(orphan.parent().unwrap()).unwrap();
    std::fs::write(&orphan, b"x").unwrap();
    let mut h = Handler::start_in(redis, dir, &[]).await;
    h.program.terminate(Duration::from_secs(10));
    let logs = log_text(&h.dir);
    assert!(logs.lines().any(|l| l.contains("WARN") && l.contains(&orphan.display().to_string())), "{logs}");
    assert!(orphan.exists());
}

// DB-INI-09
#[tokio::test]
#[cfg_attr(not(feature = "failpoints"), ignore = "needs --features failpoints")]
async fn db_ini_09_ready_only_when_started() {
    let redis = RedisTestServer::start().await;
    let dir = TempDataDir::new();
    let mut env = dir.env(&redis.url());
    env.push(("FAILPOINTS".into(), "db.migrate=sleep(3000)".into()));
    let _p = ProgramUnderTest::spawn(&binary(), P, &env);
    let bus = targetscope_common::bus::Bus::connect(&redis.url(), "probe").await.unwrap();
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert!(bus.ping(P, Duration::from_millis(1500)).await.is_err());
    let r = bus.ping(P, Duration::from_secs(10)).await.unwrap();
    assert_eq!(r.program, P);
}

// DB-INI-10
#[tokio::test]
async fn db_ini_10_reload_event_after_start() {
    let redis = RedisTestServer::start().await;
    let dir = TempDataDir::new();
    let mut probe = targetscope_testkit::EventProbe::start(&redis.url(), P).await;
    let _h = Handler::start_in(redis, dir, &[]).await;
    assert_eq!(changes(&mut probe, 1).await, ["all/null/reload"]);
}
