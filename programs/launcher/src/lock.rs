// doc/programs/launcher.md#startup-sequence

use std::fs::{File, OpenOptions};
use std::path::PathBuf;

pub struct LauncherLock {
    _file: File,
}

pub fn lock_path() -> PathBuf {
    let base = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join(format!("targetscope-{}", nix::unistd::getuid().as_raw())));
    base.join("targetscope").join("launcher.lock")
}

pub fn acquire() -> Result<LauncherLock, String> {
    let path = lock_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    let file = OpenOptions::new().create(true).truncate(false).write(true).open(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    fs4::FileExt::try_lock(&file).map_err(|_| "another TargetScope launcher is running".to_string())?;
    Ok(LauncherLock { _file: file })
}
