//! `TempDataDir`: temporary data, image, staging and export folders; optionally a small
//! tmpfs ("disk full") and a second mount ("other file system").

use std::path::PathBuf;

pub struct TempDataDir {
    pub data_dir: PathBuf,
    pub image_dir: PathBuf,
    pub staging_dir: PathBuf,
    pub export_dir: PathBuf,
    pub log_dir: PathBuf,
    // TODO: tempfile::TempDir guard
}

impl TempDataDir {
    pub fn new() -> Self {
        todo!()
    }

    /// A folder on a small tmpfs of the given size, for "disk full" tests.
    pub fn small_tmpfs(&self, size_bytes: u64) -> PathBuf {
        let _ = size_bytes;
        todo!()
    }

    /// A folder on another file system than the staging folder.
    pub fn other_file_system(&self) -> PathBuf {
        todo!()
    }

    /// The TARGETSCOPE_* bootstrap variables for a program under test.
    pub fn env(&self, redis_url: &str) -> Vec<(String, String)> {
        let _ = redis_url;
        todo!()
    }
}

impl Default for TempDataDir {
    fn default() -> Self {
        Self::new()
    }
}
