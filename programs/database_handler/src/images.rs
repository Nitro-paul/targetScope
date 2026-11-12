// doc/programs/database_handler.md#imagestore

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::error::{HandlerError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageKind {
    Jpeg,
    Png,
}

impl ImageKind {
    pub fn ext(&self) -> &'static str {
        match self {
            ImageKind::Jpeg => "jpg",
            ImageKind::Png => "png",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ImageStore {
    pub image_dir: PathBuf,
    pub staging_dir: PathBuf,
}

// doc/interfaces.md#staging-folder
pub fn is_plain_name(file: &str) -> bool {
    !file.is_empty() && !file.contains('/') && !file.contains('\\') && file != "." && file != ".." && !file.contains('\0')
}

// doc/interfaces.md#database_handler
pub fn sniff(path: &Path) -> std::io::Result<Option<ImageKind>> {
    let mut head = [0u8; 8];
    let n = fs::File::open(path)?.read(&mut head)?;
    Ok(match &head[..n] {
        [0xFF, 0xD8, 0xFF, ..] => Some(ImageKind::Jpeg),
        [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A] => Some(ImageKind::Png),
        _ => None,
    })
}

impl ImageStore {
    pub fn check_staged(&self, file: &str) -> Result<ImageKind> {
        if !is_plain_name(file) {
            return Err(HandlerError::validation(format!("staged_file must be a plain file name: {file}")));
        }
        let path = self.staging_dir.join(file);
        if !path.is_file() {
            return Err(HandlerError::not_found(format!("staged file {file}")));
        }
        sniff(&path)?.ok_or_else(|| HandlerError::validation(format!("{file} is not a JPEG or PNG image")))
    }

    pub fn move_in(&self, file: &str, sub: &str, name: &str) -> Result<String> {
        let dir = self.image_dir.join(sub);
        fs::create_dir_all(&dir)?;
        fs::rename(self.staging_dir.join(file), dir.join(name))?;
        Ok(format!("{sub}/{name}"))
    }

    pub fn move_back(&self, image_path: &str, file: &str) -> Result<()> {
        fs::rename(self.image_dir.join(image_path), self.staging_dir.join(file))?;
        Ok(())
    }

    pub fn delete(&self, image_path: &str) {
        if let Err(e) = fs::remove_file(self.image_dir.join(image_path)) {
            tracing::warn!(image_path, error = %e, "image file could not be deleted");
        }
    }

    pub fn set_image_dir(&mut self, new_dir: PathBuf) {
        self.image_dir = new_dir;
    }
}

// doc/programs/database_handler.md#image-handling-per-request
pub struct Moved {
    pub staged: String,
    pub image_path: String,
}

// doc/programs/database_handler.md#request-processing
pub fn undo(store: &ImageStore, moved: &[Moved]) {
    for m in moved.iter().rev() {
        if let Err(e) = store.move_back(&m.image_path, &m.staged) {
            tracing::error!(image_path = %m.image_path, error = %e, "moving image back failed");
        }
    }
}
