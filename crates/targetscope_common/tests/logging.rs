// doc/testing/shared_libraries.md#logging-integration

use std::sync::Arc;
use std::time::{Duration, SystemTime};

use targetscope_common::config::BootstrapConfig;
use targetscope_common::logging::{subscriber, DailyFile};
use targetscope_testkit::FakeClock;

fn config(dir: &std::path::Path, level: &str) -> BootstrapConfig {
    BootstrapConfig::from_vars(|n| match n {
        "TARGETSCOPE_LOG_DIR" => Some(dir.display().to_string()),
        "TARGETSCOPE_LOG" => Some(level.to_string()),
        _ => None,
    })
}

// LIB-LOG-01
#[test]
fn lib_log_01_two_outputs() {
    let dir = tempfile::tempdir().unwrap();
    let clock = Arc::new(targetscope_common::clock::SystemClock);
    tracing::subscriber::with_default(subscriber("test", &config(dir.path(), "info"), clock), || tracing::info!("hello file"));
    let text = std::fs::read_to_string(dir.path().join("test.log")).unwrap();
    assert!(text.contains("hello file"));
}

// LIB-LOG-02
#[test]
fn lib_log_02_daily_file() {
    use std::io::Write;
    use tracing_subscriber::fmt::MakeWriter;
    let dir = tempfile::tempdir().unwrap();
    let start = SystemTime::UNIX_EPOCH + Duration::from_secs(1_794_393_600);
    let clock = Arc::new(FakeClock::new(start));
    let file = DailyFile::new(dir.path().to_path_buf(), "test", clock.clone());
    writeln!(file.make_writer(), "day one").unwrap();
    clock.advance(Duration::from_secs(86_400));
    writeln!(file.make_writer(), "day two").unwrap();
    let names: Vec<String> = std::fs::read_dir(dir.path()).unwrap().map(|e| e.unwrap().file_name().to_string_lossy().into_owned()).collect();
    assert_eq!(names.len(), 2, "{names:?}");
    assert!(std::fs::read_to_string(dir.path().join("test.log")).unwrap().contains("day two"));
    let old = names.iter().find(|n| n.starts_with("test.log.")).unwrap();
    assert!(std::fs::read_to_string(dir.path().join(old)).unwrap().contains("day one"));
}

// LIB-LOG-03
#[test]
fn lib_log_03_request_id_in_every_line() {
    let dir = tempfile::tempdir().unwrap();
    let clock = Arc::new(targetscope_common::clock::SystemClock);
    let id = uuid::Uuid::new_v4();
    tracing::subscriber::with_default(subscriber("test", &config(dir.path(), "info"), clock), || {
        let span = tracing::info_span!("request", request_id = %id);
        let _g = span.enter();
        for i in 0..3 {
            tracing::info!(i, "handler line");
        }
    });
    let text = std::fs::read_to_string(dir.path().join("test.log")).unwrap();
    let lines: Vec<&str> = text.lines().filter(|l| l.contains("handler line")).collect();
    assert_eq!(lines.len(), 3);
    assert!(lines.iter().all(|l| l.contains(&id.to_string())));
}

// LIB-LOG-04
#[test]
fn lib_log_04_level() {
    let dir = tempfile::tempdir().unwrap();
    let clock = Arc::new(targetscope_common::clock::SystemClock);
    tracing::subscriber::with_default(subscriber("test", &config(dir.path(), "warn"), clock), || {
        tracing::info!("info line");
        tracing::warn!("warning line");
    });
    let text = std::fs::read_to_string(dir.path().join("test.log")).unwrap();
    assert!(text.contains("warning line") && !text.contains("info line"));
}
