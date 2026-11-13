// doc/testing/shared_libraries.md#bootstrap-configuration-unit

use std::path::PathBuf;

use targetscope_common::config::BootstrapConfig;

// LIB-CFG-01
#[test]
fn lib_cfg_01_all_variables_set() {
    let c = BootstrapConfig::from_vars(|name| {
        Some(
            match name {
                "TARGETSCOPE_REDIS_URL" => "redis://127.0.0.1:7000/",
                "TARGETSCOPE_DATA_DIR" => "/srv/data",
                "TARGETSCOPE_DB_PATH" => "/srv/db.sqlite",
                "TARGETSCOPE_LOG_DIR" => "/srv/logs",
                "TARGETSCOPE_LOG" => "debug",
                _ => return None,
            }
            .to_string(),
        )
    });
    assert_eq!(c.redis_url, "redis://127.0.0.1:7000/");
    assert_eq!(c.data_dir, PathBuf::from("/srv/data"));
    assert_eq!(c.db_path, PathBuf::from("/srv/db.sqlite"));
    assert_eq!(c.log_dir, PathBuf::from("/srv/logs"));
    assert_eq!(c.log_level, "debug");
}

// LIB-CFG-02
#[test]
fn lib_cfg_02_defaults() {
    let home = PathBuf::from(std::env::var("HOME").unwrap());
    let c = BootstrapConfig::from_vars(|_| None);
    assert_eq!(c.redis_url, "redis://127.0.0.1:6379/");
    assert_eq!(c.data_dir, home.join(".local/share/targetscope"));
    assert_eq!(c.db_path, home.join(".local/share/targetscope/targetscope.db"));
    assert_eq!(c.log_dir, home.join(".local/state/targetscope/logs"));
    assert_eq!(c.log_level, "info");
}
