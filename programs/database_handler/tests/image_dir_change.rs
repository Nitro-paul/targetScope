// doc/testing/database_handler.md#image-folder-change-integration

mod common;

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use common::*;
use serde_json::{json, Value};
use targetscope_common::messages::{ResponseEnvelope, Status};
use targetscope_testkit::fixtures::ReferenceIds;
use uuid::Uuid;

async fn sessions_with_images(h: &Handler, ids: ReferenceIds, sessions: usize, papers: usize) -> Vec<String> {
    let mut paths = Vec::new();
    for _ in 0..sessions {
        let images: Vec<Value> = (1..=papers).map(|n| json!({ "paper_no": n, "staged_file": h.stage("paper.jpg") })).collect();
        let mut p = cs_ref(ids, None);
        p["paper_count"] = json!(papers);
        p["images"] = json!(images);
        let r = h.ok("create_session", p).await;
        paths.extend(r["images"].as_array().unwrap().iter().map(|i| i["image_path"].as_str().unwrap().to_string()));
    }
    paths
}

fn stored_paths(h: &Handler) -> Vec<String> {
    let mut v: Vec<String> = h
        .db()
        .prepare("SELECT image_path FROM session_images ORDER BY image_path")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    v.sort();
    v
}

fn image_files(dir: &Path) -> Vec<String> {
    let mut v = Vec::new();
    for sub in ["devices", "sessions"] {
        v.extend(files_in(&dir.join(sub)).into_iter().map(|f| format!("{sub}/{f}")));
    }
    v.sort();
    v
}

async fn change(h: &Handler, path: &Path, move_images: bool, timeout: Duration) -> ResponseEnvelope {
    let payload = json!({ "path": path.display().to_string(), "move_images": move_images });
    h.bus.try_send(P, Uuid::new_v4(), "change_image_dir", payload, timeout).await.expect("change_image_dir response")
}

fn spawn_change(h: &Handler, path: PathBuf, move_images: bool) -> tokio::task::JoinHandle<Option<ResponseEnvelope>> {
    let url = h.redis.url();
    tokio::spawn(async move {
        let bus = targetscope_testkit::TestBus::connect(&url).await;
        let payload = json!({ "path": path.display().to_string(), "move_images": move_images });
        bus.try_send(P, Uuid::new_v4(), "change_image_dir", payload, Duration::from_secs(60)).await
    })
}

async fn wait_copied(new_dir: &Path, n: usize) {
    let deadline = Instant::now() + Duration::from_secs(20);
    while image_files(new_dir).len() < n {
        assert!(Instant::now() < deadline, "copy didn't start");
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

// DB-DIR-01
#[tokio::test]
async fn db_dir_01_without_moving() {
    let h = Handler::start().await;
    let ids = h.refs().await;
    let mut paths = sessions_with_images(&h, ids, 1, 10).await;
    paths.sort();
    let new_dir = h.dir.root.join("images_new");
    let r = change(&h, &new_dir, false, Duration::from_secs(10)).await;
    assert_eq!(r.status, Status::Ok, "{:?}", r.error);
    assert_eq!(r.payload.unwrap()["images"], Value::Null);
    assert_eq!(h.setting("image_dir").await["value"], new_dir.display().to_string());
    assert_eq!(image_files(&h.dir.image_dir), paths);
    let later = h.create_session(ids).await;
    assert!(new_dir.join(later["images"][0]["image_path"].as_str().unwrap()).exists());
}

// DB-DIR-02
#[tokio::test]
async fn db_dir_02_with_moving() {
    let h = Handler::start().await;
    let ids = h.refs().await;
    let paths = sessions_with_images(&h, ids, 5, 10).await;
    let before = stored_paths(&h);
    let new_dir = h.dir.root.join("images_new");
    let r = change(&h, &new_dir, true, Duration::from_secs(60)).await;
    assert_eq!(r.status, Status::Ok, "{:?}", r.error);
    assert_eq!(r.payload.unwrap()["images"], json!({ "moved": 50, "failed": [] }));
    let mut expected = paths.clone();
    expected.sort();
    assert_eq!(image_files(&new_dir), expected);
    assert!(image_files(&h.dir.image_dir).is_empty());
    assert_eq!(stored_paths(&h), before);
}

// DB-DIR-03
#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "failpoints"), ignore = "needs --features failpoints")]
async fn db_dir_03_saving_during_the_copy() {
    let mut h = Handler::start().await;
    let ids = h.refs().await;
    sessions_with_images(&h, ids, 1, 10).await;
    h.restart_with(&[("FAILPOINTS", "db.image_dir_copy=sleep(1000)")]).await;
    let new_dir = h.dir.root.join("images_new");
    let pending = spawn_change(&h, new_dir.clone(), true);
    wait_copied(&new_dir, 1).await;
    let started = Instant::now();
    let r = h.create_session(ids).await;
    assert!(started.elapsed() < Duration::from_secs(5));
    let path = r["images"][0]["image_path"].as_str().unwrap().to_string();
    let done = pending.await.unwrap().expect("change_image_dir response");
    assert_eq!(done.status, Status::Ok, "{:?}", done.error);
    assert!(new_dir.join(&path).exists());
    assert!(!h.dir.image_dir.join(&path).exists());
}

// DB-DIR-04
#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "failpoints"), ignore = "needs --features failpoints")]
async fn db_dir_04_deleting_during_the_copy() {
    let mut h = Handler::start().await;
    let ids = h.refs().await;
    let first = h.create_session(ids).await;
    sessions_with_images(&h, ids, 1, 10).await;
    let path = first["images"][0]["image_path"].as_str().unwrap().to_string();
    h.restart_with(&[("FAILPOINTS", "db.image_dir_copy=sleep(1000)")]).await;
    let new_dir = h.dir.root.join("images_new");
    let pending = spawn_change(&h, new_dir.clone(), true);
    let deadline = Instant::now() + Duration::from_secs(20);
    while !new_dir.join(&path).exists() {
        assert!(Instant::now() < deadline, "first image not copied");
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    h.ok("delete_session", json!({ "id": first["id"] })).await;
    let done = pending.await.unwrap().expect("change_image_dir response");
    assert_eq!(done.status, Status::Ok, "{:?}", done.error);
    assert!(!new_dir.join(&path).exists());
    assert!(!h.dir.image_dir.join(&path).exists());
}

// DB-DIR-05
#[tokio::test]
async fn db_dir_05_file_that_can_t_be_read() {
    let h = Handler::start().await;
    let ids = h.refs().await;
    let paths = sessions_with_images(&h, ids, 1, 5).await;
    let unreadable = h.dir.image_dir.join(&paths[2]);
    std::fs::set_permissions(&unreadable, std::fs::Permissions::from_mode(0o000)).unwrap();
    let new_dir = h.dir.root.join("images_new");
    let r = change(&h, &new_dir, true, Duration::from_secs(60)).await;
    std::fs::set_permissions(&unreadable, std::fs::Permissions::from_mode(0o644)).unwrap();
    assert_eq!(r.status, Status::Ok, "{:?}", r.error);
    assert_eq!(r.payload.unwrap()["images"], json!({ "moved": 4, "failed": [paths[2]] }));
    assert!(unreadable.exists());
    assert_eq!(image_files(&new_dir).len(), 4);
}

// DB-DIR-06
#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "failpoints"), ignore = "needs --features failpoints")]
async fn db_dir_06_second_change_while_one_runs() {
    let mut h = Handler::start().await;
    let ids = h.refs().await;
    sessions_with_images(&h, ids, 1, 5).await;
    h.restart_with(&[("FAILPOINTS", "db.image_dir_copy=sleep(1000)")]).await;
    let new_dir = h.dir.root.join("images_new");
    let pending = spawn_change(&h, new_dir.clone(), true);
    wait_copied(&new_dir, 1).await;
    let second = change(&h, &h.dir.root.join("images_other"), true, Duration::from_secs(10)).await;
    assert_eq!(second.error.unwrap().code.as_str(), "invalid_state");
    assert_eq!(pending.await.unwrap().unwrap().status, Status::Ok);
}

// DB-DIR-07
#[tokio::test]
async fn db_dir_07_other_file_system_than_the_staging_folder() {
    let h = Handler::start().await;
    let Some(other) = h.dir.other_file_system() else {
        eprintln!("no second file system available; DB-DIR-07 not checked");
        return;
    };
    let r = change(&h, &other.join("images"), true, Duration::from_secs(10)).await;
    assert_eq!(r.error.unwrap().code.as_str(), "validation_failed");
    assert_eq!(h.setting("image_dir").await["value"], h.dir.image_dir.display().to_string());
    let _ = std::fs::remove_dir_all(other);
}

// DB-DIR-08
#[tokio::test]
async fn db_dir_08_folder_can_t_be_written() {
    let h = Handler::start().await;
    let locked = h.dir.root.join("locked");
    std::fs::create_dir(&locked).unwrap();
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o555)).unwrap();
    let r = change(&h, &locked.join("images"), true, Duration::from_secs(10)).await;
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert_eq!(r.error.unwrap().code.as_str(), "validation_failed");
}

// DB-DIR-09
#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "failpoints"), ignore = "needs --features failpoints")]
async fn db_dir_09_crash_during_the_copy() {
    let mut h = Handler::start().await;
    let ids = h.refs().await;
    let paths = sessions_with_images(&h, ids, 1, 10).await;
    h.restart_with(&[("FAILPOINTS", "db.image_dir_copy=sleep(1000)")]).await;
    let new_dir = h.dir.root.join("images_new");
    let pending = spawn_change(&h, new_dir.clone(), true);
    wait_copied(&new_dir, 2).await;
    h.program.kill();
    pending.abort();
    let db = h.db();
    let setting = || db.query_row("SELECT value FROM settings WHERE name = 'image_dir'", [], |r| r.get::<_, String>(0)).unwrap();
    assert_eq!(setting(), h.dir.image_dir.display().to_string());
    for p in &paths {
        assert!(std::fs::read(h.dir.image_dir.join(p)).is_ok(), "{p}");
    }
    h.restart().await;
    let deadline = Instant::now() + Duration::from_secs(30);
    while setting() != new_dir.display().to_string() {
        assert!(Instant::now() < deadline, "the pending change didn't run again");
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let deadline = Instant::now() + Duration::from_secs(30);
    while !image_files(&h.dir.image_dir).is_empty() {
        assert!(Instant::now() < deadline, "old copies not removed");
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let mut expected = paths.clone();
    expected.sort();
    assert_eq!(image_files(&new_dir), expected);
    for p in &paths {
        assert_eq!(
            std::fs::read(new_dir.join(p)).unwrap(),
            std::fs::read(targetscope_testkit::fixtures::testdata("images/paper.jpg")).unwrap()
        );
    }
}

// DB-DIR-10
#[tokio::test]
async fn db_dir_10_reset_with_moving() {
    let h = Handler::start().await;
    let ids = h.refs().await;
    let elsewhere = h.dir.root.join("images_elsewhere");
    assert_eq!(change(&h, &elsewhere, true, Duration::from_secs(60)).await.status, Status::Ok);
    let paths = sessions_with_images(&h, ids, 1, 5).await;
    assert_eq!(image_files(&elsewhere).len(), 5);
    let r = h.ok("reset_settings", json!({ "names": ["image_dir"], "move_images": true })).await;
    assert_eq!(r["images"], json!({ "moved": 5, "failed": [] }));
    let default_dir = h.dir.data_dir.join("images");
    assert_eq!(h.setting("image_dir").await["value"], default_dir.display().to_string());
    let mut expected = paths;
    expected.sort();
    assert_eq!(image_files(&default_dir), expected);
    assert!(image_files(&elsewhere).is_empty());
}
