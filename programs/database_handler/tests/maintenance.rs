// doc/testing/database_handler.md#maintenance-integration

use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use database_handler::db::Database;
use database_handler::db_worker::spawn_db_worker;
use database_handler::images::ImageStore;
use database_handler::maintenance::run_periodically;
use targetscope_common::clock::Clock;
use targetscope_testkit::{FakeClock, TempDataDir};

const DAY: i64 = 86_400;

struct Maintenance {
    dir: TempDataDir,
    clock: Arc<FakeClock>,
    now_s: i64,
}

impl Maintenance {
    fn start(prepare: impl FnOnce(&rusqlite::Connection, i64, &Path)) -> Self {
        let dir = TempDataDir::new();
        let start = SystemTime::now();
        let now_s = start.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as i64;
        let db = Database::open(&dir.db_path(), &dir.data_dir).unwrap();
        prepare(&db.conn, now_s, &dir.staging_dir);
        let clock = Arc::new(FakeClock::new(start));
        let images = ImageStore { image_dir: dir.image_dir.clone(), staging_dir: dir.staging_dir.clone() };
        let tx = spawn_db_worker(db, images, dir.data_dir.clone(), clock.clone() as Arc<dyn Clock>);
        tokio::spawn(run_periodically(tx, clock.clone() as Arc<dyn Clock>));
        Maintenance { dir, clock, now_s }
    }

    fn conn(&self) -> rusqlite::Connection {
        rusqlite::Connection::open(self.dir.db_path()).unwrap()
    }

    fn request_ids(&self) -> Vec<String> {
        let c = self.conn();
        let mut stmt = c.prepare("SELECT request_id FROM processed_requests ORDER BY request_id").unwrap();
        stmt.query_map([], |r| r.get(0)).unwrap().map(Result::unwrap).collect()
    }

    async fn until(&self, what: &str, done: impl Fn(&Self) -> bool) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while !done(self) {
            assert!(Instant::now() < deadline, "{what}");
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }
}

fn insert(c: &rusqlite::Connection, id: &str, processed_at: i64) {
    c.execute(
        "INSERT INTO processed_requests (request_id, type, processed_at, response) VALUES (?1, 'save_shooter', ?2, '{}')",
        rusqlite::params![id, processed_at],
    )
    .unwrap();
}

fn staged(dir: &Path, name: &str, age_hours: u64) {
    let path = dir.join(name);
    std::fs::write(&path, b"x").unwrap();
    let f = std::fs::File::options().write(true).open(&path).unwrap();
    f.set_modified(SystemTime::now() - Duration::from_secs(age_hours * 3600)).unwrap();
}

// DB-MNT-01
#[tokio::test(flavor = "multi_thread")]
async fn db_mnt_01_purge_processed_requests() {
    let m = Maintenance::start(|c, now, _| {
        insert(c, "a-29-days", now - 29 * DAY);
        insert(c, "b-31-days", now - 31 * DAY);
    });
    m.until("31-day-old row not purged", |m| m.request_ids() == ["a-29-days"]).await;
}

// DB-MNT-02
#[tokio::test(flavor = "multi_thread")]
async fn db_mnt_02_retention_setting() {
    let m = Maintenance::start(|c, now, _| {
        insert(c, "a-2-days", now - 2 * DAY);
        insert(c, "b-12-hours", now - DAY / 2);
    });
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(m.request_ids(), ["a-2-days", "b-12-hours"]);
    m.conn().execute("UPDATE settings SET value = '1' WHERE name = 'processed_requests_retention_days'", []).unwrap();
    m.clock.advance(Duration::from_secs(3600));
    m.until("row older than 1 day not purged", |m| m.request_ids() == ["b-12-hours"]).await;
}

// DB-MNT-03
#[tokio::test(flavor = "multi_thread")]
async fn db_mnt_03_staging_folder() {
    let m = Maintenance::start(|_, _, staging| {
        staged(staging, "upload_new.jpg", 23);
        staged(staging, "upload_old.jpg", 25);
        staged(staging, "export_new.zip.part", 23);
        staged(staging, "export_old.zip.part", 25);
        std::fs::create_dir(staging.join("old_folder")).unwrap();
    });
    let files = |m: &Maintenance| {
        let mut v: Vec<String> = std::fs::read_dir(&m.dir.staging_dir)
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        v.sort();
        v
    };
    m.until("old staged files not deleted", |m| files(m) == ["export_new.zip.part", "old_folder", "upload_new.jpg"]).await;
}

// DB-MNT-04
#[tokio::test(flavor = "multi_thread")]
async fn db_mnt_04_runs_every_hour() {
    let m = Maintenance::start(|_, _, _| {});
    tokio::time::sleep(Duration::from_millis(300)).await;
    for hour in 1..=3 {
        let id = format!("old-{hour}");
        insert(&m.conn(), &id, m.now_s - 40 * DAY);
        m.clock.advance(Duration::from_secs(1800));
        tokio::time::sleep(Duration::from_millis(300)).await;
        assert_eq!(m.request_ids(), [id.clone()], "a run after half an hour");
        m.clock.advance(Duration::from_secs(1800));
        m.until("no run after an hour", |m| m.request_ids().is_empty()).await;
    }
}
