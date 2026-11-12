//! `Fixtures`: loads the reference data (doc/testing/overview.md#reference-data) into a
//! database, the image folder or the staging folder.

use std::path::{Path, PathBuf};

/// Copies a file from testdata/images/ into the staging folder; returns its staged name.
pub fn stage_image(staging_dir: &Path, name: &str) -> String {
    let _ = (staging_dir, name);
    todo!()
}

/// Path of a reference file below testdata/.
pub fn testdata(rel: &str) -> PathBuf {
    crate::testdata_dir().join(rel)
}

/// Creates TT-REF, SH-REF and DEV-REF through database_handler; returns their ids.
pub async fn create_reference_records(redis_url: &str) -> ReferenceIds {
    let _ = redis_url;
    todo!()
}

pub struct ReferenceIds {
    pub target_type_id: i64,
    pub shooter_id: i64,
    pub device_id: i64,
}

/// Saves a session set from testdata/sessions/<name>.json (e.g. FILTER-SET, EXPORT-SET).
pub async fn load_session_set(redis_url: &str, name: &str) {
    let _ = (redis_url, name);
    todo!()
}
