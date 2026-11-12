//! Single-instance lock `$XDG_RUNTIME_DIR/targetscope/launcher.lock` (fs4).

use std::fs::File;

pub struct LauncherLock {
    _file: File,
}

/// Err with a message if another launcher holds the lock.
pub fn acquire() -> Result<LauncherLock, String> {
    todo!()
}
