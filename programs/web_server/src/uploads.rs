//! Uploads (doc/programs/web_server.md#uploads): multipart with one file, streamed to the
//! staging folder as upload_<uuid>.<ext> (temporary name, then rename). Images JPEG/PNG up to
//! 20 MB, JSON up to 100 MB, ZIP up to 2 GB (magic bytes); larger → 413 too_large.

use axum::extract::Multipart;

use crate::errors::ApiError;

pub const MAX_IMAGE_BYTES: u64 = 20 * 1024 * 1024;
pub const MAX_JSON_BYTES: u64 = 100 * 1024 * 1024;
pub const MAX_ZIP_BYTES: u64 = 2 * 1024 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UploadKind {
    Image,
    Import,
}

pub struct Upload {
    pub staged_file: String,
    /// Other form fields (e.g. corners of a photo upload)
    pub fields: Vec<(String, String)>,
}

pub async fn receive(
    multipart: Multipart,
    kind: UploadKind,
    staging_dir: &std::path::Path,
) -> Result<Upload, ApiError> {
    let _ = (multipart, kind, staging_dir);
    todo!()
}
