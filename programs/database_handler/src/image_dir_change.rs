//! change_image_dir (doc/programs/database_handler.md#image-folder-change):
//! 1. copy in the background, 2. switch in one short DbWorker command, 3. clean up.
//! Only one change at a time; a second one gets invalid_state.

use std::path::{Path, PathBuf};

use crate::error::HandlerError;

pub struct MoveReport {
    pub moved: u64,
    pub failed: Vec<String>,
}

/// Step 1: copies all files of devices/ and sessions/ into the new folder.
pub fn copy_all(old_dir: &Path, new_dir: &Path) -> Result<Vec<PathBuf>, HandlerError> {
    let _ = (old_dir, new_dir);
    todo!()
}

/// Step 3: deletes the copied files in the old folder.
pub fn clean_up(old_dir: &Path, copied: &[PathBuf]) -> MoveReport {
    let _ = (old_dir, copied);
    todo!()
}
