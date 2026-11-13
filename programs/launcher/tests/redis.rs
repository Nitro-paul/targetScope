// doc/testing/launcher.md#redis-integration

// LA-RDS-01
#[tokio::test]
#[ignore = "not implemented"]
async fn la_rds_01_options() {
    todo!("LA-RDS-01")
}

// LA-RDS-02
#[tokio::test]
#[ignore = "not implemented"]
async fn la_rds_02_port_in_use() {
    todo!("LA-RDS-02")
}

// LA-RDS-03
#[tokio::test]
#[ignore = "not implemented"]
async fn la_rds_03_redis_doesn_t_answer() {
    todo!("LA-RDS-03")
}

// LA-RDS-04
#[tokio::test]
#[ignore = "not implemented"]
async fn la_rds_04_redis_crashes() {
    todo!("LA-RDS-04")
}

// LA-RDS-05
#[tokio::test]
#[ignore = "not implemented"]
async fn la_rds_05_clean_stop() {
    todo!("LA-RDS-05")
}

// LA-RDS-06
#[test]
fn la_rds_06_redis_from_the_deps_folder() {
    use launcher::redis_server::find_binary_in;
    let home = tempfile::tempdir().unwrap();
    let bin = home.path().join(".local/opt/targetscope-deps/bin");
    std::fs::create_dir_all(&bin).unwrap();
    std::fs::write(bin.join("redis-server"), b"").unwrap();
    let empty = tempfile::tempdir().unwrap();
    let path = Some(empty.path().as_os_str().to_owned());
    assert_eq!(find_binary_in("redis-server", path.clone(), None, Some(home.path().into())), Some(bin.join("redis-server")));
    let deps = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(deps.path().join("bin")).unwrap();
    std::fs::write(deps.path().join("bin/redis-server"), b"").unwrap();
    assert_eq!(
        find_binary_in("redis-server", path.clone(), Some(deps.path().into()), Some(home.path().into())),
        Some(deps.path().join("bin/redis-server"))
    );
    std::fs::write(empty.path().join("redis-server"), b"").unwrap();
    assert_eq!(
        find_binary_in("redis-server", path, Some(deps.path().into()), Some(home.path().into())),
        Some(empty.path().join("redis-server"))
    );
}

// LA-RDS-07
#[test]
fn la_rds_07_redis_not_found() {
    use launcher::redis_server::find_binary_in;
    let empty = tempfile::tempdir().unwrap();
    let path = Some(empty.path().as_os_str().to_owned());
    assert_eq!(find_binary_in("redis-server", path, None, Some(empty.path().into())), None);
    assert_eq!(find_binary_in("/nonexistent/redis-server", None, None, None), None);
}
