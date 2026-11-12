//! ImageStore: staging -> image folder, delete (doc/programs/database_handler.md#imagestore).

use std::path::PathBuf;

use crate::error::HandlerError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageKind {
    Jpeg,
    Png,
}

pub struct ImageStore {
    pub image_dir: PathBuf,
    pub staging_dir: PathBuf,
}

impl ImageStore {
    /// Plain file name (else ValidationFailed), exists in staging (else NotFound),
    /// JPEG or PNG by magic bytes (else ValidationFailed).
    pub fn check_staged(&self, file: &str) -> Result<ImageKind, HandlerError> {
        let _ = file;
        todo!()
    }

    /// Atomic rename from the staging folder to `image_dir/<sub>/<name>`; returns the image_path.
    pub fn move_in(&self, file: &str, sub: &str, name: &str) -> Result<String, HandlerError> {
        let _ = (file, sub, name);
        todo!()
    }

    /// Moves a file back to the staging folder (used when a transaction fails).
    pub fn move_back(&self, image_path: &str, file: &str) -> Result<(), HandlerError> {
        let _ = (image_path, file);
        todo!()
    }

    /// Deletes a file from the image folder; a missing file is only logged.
    pub fn delete(&self, image_path: &str) {
        let _ = image_path;
        todo!()
    }

    pub fn set_image_dir(&mut self, new_dir: PathBuf) {
        self.image_dir = new_dir;
    }
}
