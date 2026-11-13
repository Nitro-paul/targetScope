// doc/testing/overview.md#test-kit

use std::path::PathBuf;
use std::process::Command;

pub struct TempDataDir {
    pub root: PathBuf,
    pub data_dir: PathBuf,
    pub image_dir: PathBuf,
    pub staging_dir: PathBuf,
    pub export_dir: PathBuf,
    pub log_dir: PathBuf,
    _tmp: tempfile::TempDir,
}

impl TempDataDir {
    pub fn new() -> Self {
        let tmp = tempfile::tempdir().expect("tempdir");
        let root = tmp.path().to_path_buf();
        let data_dir = root.join("data");
        let d = TempDataDir {
            image_dir: data_dir.join("images"),
            staging_dir: data_dir.join("staging"),
            export_dir: root.join("exports"),
            log_dir: root.join("logs"),
            data_dir,
            root,
            _tmp: tmp,
        };
        for dir in [&d.data_dir, &d.image_dir, &d.staging_dir, &d.export_dir, &d.log_dir] {
            std::fs::create_dir_all(dir).unwrap();
        }
        d
    }

    pub fn db_path(&self) -> PathBuf {
        self.data_dir.join("targetscope.db")
    }

    // doc/shared_libraries.md#bootstrap-configuration
    pub fn env(&self, redis_url: &str) -> Vec<(String, String)> {
        vec![
            ("TARGETSCOPE_REDIS_URL".into(), redis_url.into()),
            ("TARGETSCOPE_DATA_DIR".into(), self.data_dir.display().to_string()),
            ("TARGETSCOPE_DB_PATH".into(), self.db_path().display().to_string()),
            ("TARGETSCOPE_LOG_DIR".into(), self.log_dir.display().to_string()),
            ("TARGETSCOPE_LOG".into(), "debug".into()),
        ]
    }

    // doc/testing/overview.md#test-kit
    pub fn small_tmpfs(&self, size_bytes: u64) -> Option<PathBuf> {
        let dir = self.root.join(format!("tmpfs_{size_bytes}"));
        std::fs::create_dir_all(&dir).ok()?;
        let ok = Command::new("mount")
            .args(["-t", "tmpfs", "-o", &format!("size={size_bytes}"), "tmpfs"])
            .arg(&dir)
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        ok.then_some(dir)
    }

    // doc/testing/overview.md#test-kit
    pub fn other_file_system(&self) -> Option<PathBuf> {
        ["/dev/shm", "/run/user"]
            .iter()
            .map(PathBuf::from)
            .find(|p| p.is_dir() && same_device(p, &self.root) == Some(false))
            .and_then(|p| tempfile::Builder::new().prefix("targetscope_").tempdir_in(p).ok())
            .map(|t| t.keep())
    }
}

pub fn same_device(a: &std::path::Path, b: &std::path::Path) -> Option<bool> {
    use std::os::unix::fs::MetadataExt;
    Some(std::fs::metadata(a).ok()?.dev() == std::fs::metadata(b).ok()?.dev())
}

impl Default for TempDataDir {
    fn default() -> Self {
        Self::new()
    }
}
