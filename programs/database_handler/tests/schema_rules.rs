// doc/testing/database_handler.md#schema-rules-unit-sql

use database_handler::db::Database;
use rusqlite::{params, Connection};

fn db() -> Database {
    Database::open_in_memory(std::path::Path::new("/tmp/data")).unwrap()
}

const TT_REF: &str = "INSERT INTO target_types (uuid, name, paper_width_mm, paper_height_mm, center_x_mm, center_y_mm, \
    inner_white_radius_mm, inner_white_score, black_radius_mm, black_score, outer_radius_mm, outer_score) \
    VALUES (?1, ?2, 200, 200, 100, 100, 5, 12, 30, 7, 90, 1)";

struct Fixture {
    tt: i64,
    dev: i64,
    sh: i64,
    session: i64,
}

fn fixture(c: &Connection) -> Fixture {
    c.execute(TT_REF, params!["tt-1", "TT-REF"]).unwrap();
    let tt = c.last_insert_rowid();
    c.execute("INSERT INTO devices (uuid, name, bullet_diameter_mm) VALUES ('d-1', 'Test pistol', 4.0)", []).unwrap();
    let dev = c.last_insert_rowid();
    c.execute("INSERT INTO shooters (uuid, name) VALUES ('s-1', 'Alex')", []).unwrap();
    let sh = c.last_insert_rowid();
    c.execute(
        "INSERT INTO sessions (uuid, saved_at, score, grouping_score, target_type_id, shooter_id, device_id, shot_count, paper_count) \
         VALUES ('x-1', 1, 38, 40, ?1, ?2, ?3, 1, 1)",
        params![tt, sh, dev],
    )
    .unwrap();
    let session = c.last_insert_rowid();
    c.execute("INSERT INTO shots (session_id, shot_no, paper_no, angle_turns, radius_mm) VALUES (?1, 1, 1, 0.5, 3)", [session]).unwrap();
    c.execute("INSERT INTO session_images (session_id, paper_no, image_path) VALUES (?1, 1, 'sessions/x-1_1.jpg')", [session]).unwrap();
    Fixture { tt, dev, sh, session }
}

fn aborted(c: &Connection, sql: &str, message: &str) {
    let err = c.execute(sql, []).unwrap_err().to_string();
    assert!(err.contains(message), "{sql}: {err}");
}

// DB-SQL-01
#[test]
fn db_sql_01_sessions_are_records() {
    let d = db();
    fixture(&d.conn);
    aborted(&d.conn, "UPDATE sessions SET notes = 'x'", "record: sessions cannot be changed");
}

// DB-SQL-02
#[test]
fn db_sql_02_shots_are_records() {
    let d = db();
    fixture(&d.conn);
    aborted(&d.conn, "UPDATE shots SET radius_mm = 1", "record: shots cannot be changed");
}

// DB-SQL-03
#[test]
fn db_sql_03_session_images_are_records() {
    let d = db();
    fixture(&d.conn);
    aborted(&d.conn, "UPDATE session_images SET image_path = 'x'", "record: session images cannot be changed");
}

// DB-SQL-04
#[test]
fn db_sql_04_deleting_a_session() {
    let d = db();
    let f = fixture(&d.conn);
    d.conn.execute("DELETE FROM sessions WHERE id = ?1", [f.session]).unwrap();
    let n: i64 = d.conn.query_row("SELECT (SELECT COUNT(*) FROM shots) + (SELECT COUNT(*) FROM session_images)", [], |r| r.get(0)).unwrap();
    assert_eq!(n, 0);
}

// DB-SQL-05
#[test]
fn db_sql_05_locked_target_type() {
    let d = db();
    let f = fixture(&d.conn);
    for col in [
        "paper_width_mm = 201",
        "paper_height_mm = 201",
        "center_x_mm = 101",
        "center_y_mm = 101",
        "inner_white_radius_mm = 4",
        "inner_white_score = 13",
        "black_radius_mm = 31",
        "black_score = 6",
        "outer_radius_mm = 89",
        "outer_score = 2",
        "uuid = 'other'",
    ] {
        aborted(&d.conn, &format!("UPDATE target_types SET {col} WHERE id = {}", f.tt), "in_use: target type is used by a session");
    }
}

// DB-SQL-06
#[test]
fn db_sql_06_locked_target_type_free_columns() {
    let d = db();
    let f = fixture(&d.conn);
    d.conn.execute("UPDATE target_types SET name = 'New' WHERE id = ?1", [f.tt]).unwrap();
    d.conn.execute("UPDATE target_types SET retired = 1 WHERE id = ?1", [f.tt]).unwrap();
}

// DB-SQL-07
#[test]
fn db_sql_07_unused_target_type() {
    let d = db();
    d.conn.execute(TT_REF, params!["tt-2", "TT-2"]).unwrap();
    d.conn.execute("UPDATE target_types SET outer_radius_mm = 80 WHERE name = 'TT-2'", []).unwrap();
}

// DB-SQL-08
#[test]
fn db_sql_08_locked_device() {
    let d = db();
    let f = fixture(&d.conn);
    for col in ["bullet_diameter_mm = 4.5", "uuid = 'other'"] {
        aborted(&d.conn, &format!("UPDATE devices SET {col} WHERE id = {}", f.dev), "in_use: bullet diameter is used by a session");
    }
}

// DB-SQL-09
#[test]
fn db_sql_09_locked_device_free_columns() {
    let d = db();
    let f = fixture(&d.conn);
    for col in ["name = 'x'", "category = 'x'", "description = 'x'", "caliber = 'x'", "image_path = 'x'", "retired = 1"] {
        d.conn.execute(&format!("UPDATE devices SET {col} WHERE id = {}", f.dev), []).unwrap();
    }
}

// DB-SQL-10
#[test]
fn db_sql_10_shot_count_never_decreases() {
    let d = db();
    let f = fixture(&d.conn);
    d.conn.execute("UPDATE devices SET shot_count = 5 WHERE id = ?1", [f.dev]).unwrap();
    aborted(&d.conn, &format!("UPDATE devices SET shot_count = shot_count - 1 WHERE id = {}", f.dev), "record: shot_count cannot decrease");
    d.conn.execute("UPDATE devices SET shot_count = shot_count + 1 WHERE id = ?1", [f.dev]).unwrap();
}

// DB-SQL-11
#[test]
fn db_sql_11_shooter_names_ignore_case() {
    let d = db();
    d.conn.execute("INSERT INTO shooters (uuid, name) VALUES ('a', 'Alex')", []).unwrap();
    assert!(d.conn.execute("INSERT INTO shooters (uuid, name) VALUES ('b', 'ALEX')", []).is_err());
}

// DB-SQL-12
#[test]
fn db_sql_12_shooter_names_are_trimmed() {
    let d = db();
    assert!(d.conn.execute("INSERT INTO shooters (uuid, name) VALUES ('a', ' Alex')", []).is_err());
    assert!(d.conn.execute("INSERT INTO shooters (uuid, name) VALUES ('b', '')", []).is_err());
}

// DB-SQL-13
#[test]
fn db_sql_13_target_type_names_ignore_case() {
    let d = db();
    d.conn.execute(TT_REF, params!["a", "TT-REF"]).unwrap();
    assert!(d.conn.execute(TT_REF, params!["b", "tt-ref"]).is_err());
}

// DB-SQL-14
#[test]
fn db_sql_14_shot_ranges() {
    let d = db();
    let f = fixture(&d.conn);
    let insert = |shot_no: i64, paper_no: i64, a: f64, r: f64| {
        d.conn.execute(
            "INSERT INTO shots (session_id, shot_no, paper_no, angle_turns, radius_mm) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![f.session, shot_no, paper_no, a, r],
        )
    };
    assert!(insert(10, 1, 1.0, 1.0).is_err());
    assert!(insert(11, 1, -0.1, 1.0).is_err());
    assert!(insert(12, 1, 0.5, -0.1).is_err());
    assert!(insert(0, 1, 0.5, 1.0).is_err());
    assert!(insert(13, 0, 0.5, 1.0).is_err());
    assert!(insert(1, 1, 0.5, 1.0).is_err());
}

// DB-SQL-15
#[test]
fn db_sql_15_session_ranges() {
    let d = db();
    let f = fixture(&d.conn);
    let notes = format!("'{}'", "x".repeat(10_001));
    let cases: [(&str, &str, &str); 6] = [
        ("shot_count", "0", "paper_count"),
        ("paper_count", "0", "shot_count"),
        ("distance_m", "-1", "shot_count"),
        ("duration_s", "-1", "shot_count"),
        ("notes", &notes, "shot_count"),
        ("single_handed", "2", "shot_count"),
    ];
    for (i, (col, val, other)) in cases.iter().enumerate() {
        let extra = if col == other { String::new() } else { format!(", {other}") };
        let extra_val = if col == other { String::new() } else { ", 1".to_string() };
        let required = if *col == "shot_count" || *col == "paper_count" { String::new() } else { ", paper_count".to_string() };
        let required_val = if required.is_empty() { String::new() } else { ", 1".to_string() };
        let sql = format!(
            "INSERT INTO sessions (uuid, saved_at, score, grouping_score, target_type_id, device_id, {col}{extra}{required}) \
             VALUES ('c{i}', 1, 1, 1, {}, {}, {val}{extra_val}{required_val})",
            f.tt, f.dev
        );
        assert!(d.conn.execute(&sql, []).is_err(), "{sql}");
    }
}

// DB-SQL-16
#[test]
fn db_sql_16_target_type_rules() {
    let d = db();
    let base = [("paper_width_mm", "200"), ("paper_height_mm", "200"), ("center_x_mm", "100"), ("center_y_mm", "100"),
        ("inner_white_radius_mm", "5"), ("inner_white_score", "12"), ("black_radius_mm", "30"), ("black_score", "7"),
        ("outer_radius_mm", "90"), ("outer_score", "1")];
    let changes = [
        ("inner_white_score", "7"),
        ("black_score", "1"),
        ("outer_score", "0"),
        ("inner_white_radius_mm", "0"),
        ("inner_white_radius_mm", "30"),
        ("black_radius_mm", "90"),
        ("center_x_mm", "89"),
        ("center_x_mm", "111"),
        ("center_y_mm", "89"),
        ("center_y_mm", "111"),
        ("paper_width_mm", "0"),
    ];
    for (i, (col, val)) in changes.iter().enumerate() {
        let values: Vec<&str> = base.iter().map(|(c, v)| if c == col { *val } else { *v }).collect();
        let cols: Vec<&str> = base.iter().map(|(c, _)| *c).collect();
        let sql = format!("INSERT INTO target_types (uuid, name, {}) VALUES ('u{i}', 'T{i}', {})", cols.join(", "), values.join(", "));
        assert!(d.conn.execute(&sql, []).is_err(), "{col} = {val}");
    }
}

// DB-SQL-17
#[test]
fn db_sql_17_device_ranges() {
    let d = db();
    assert!(d.conn.execute("INSERT INTO devices (uuid, name, bullet_diameter_mm) VALUES ('a', 'x', 0)", []).is_err());
    assert!(d.conn.execute("INSERT INTO devices (uuid, name, bullet_diameter_mm, shot_count) VALUES ('b', 'x', 4, -1)", []).is_err());
}

// DB-SQL-18
#[test]
fn db_sql_18_foreign_keys() {
    let d = db();
    let f = fixture(&d.conn);
    assert!(d
        .conn
        .execute(
            "INSERT INTO sessions (uuid, saved_at, score, grouping_score, target_type_id, device_id, shot_count, paper_count) \
             VALUES ('fk', 1, 1, 1, ?1, 99999, 1, 1)",
            [f.tt],
        )
        .is_err());
    assert!(d.conn.execute("DELETE FROM devices WHERE id = ?1", [f.dev]).is_err());
    assert!(d.conn.execute("DELETE FROM target_types WHERE id = ?1", [f.tt]).is_err());
    assert!(d.conn.execute("DELETE FROM shooters WHERE id = ?1", [f.sh]).is_err());
}
