// doc/programs/web_server.md#uploads

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use axum::extract::Multipart;
use axum::http::StatusCode;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::errors::ApiError;

pub const MAX_IMAGE_BYTES: u64 = 20 * 1024 * 1024;
pub const MAX_JSON_BYTES: u64 = 100 * 1024 * 1024;
pub const MAX_ZIP_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const MAX_FIELD_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UploadKind {
    Image,
    Import,
}

pub struct Upload {
    pub staged_file: String,
    pub path: PathBuf,
    pub fields: HashMap<String, String>,
}

fn too_large() -> ApiError {
    ApiError::new(StatusCode::PAYLOAD_TOO_LARGE, "too_large", "the file is too large")
}

// doc/programs/web_server.md#uploads
fn detect(kind: UploadKind, head: &[u8]) -> Option<(&'static str, u64)> {
    match kind {
        UploadKind::Image => match head {
            [0xFF, 0xD8, 0xFF, ..] => Some(("jpg", MAX_IMAGE_BYTES)),
            [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, ..] => Some(("png", MAX_IMAGE_BYTES)),
            _ => None,
        },
        UploadKind::Import => match head {
            [b'P', b'K', 0x03, 0x04, ..] => Some(("zip", MAX_ZIP_BYTES)),
            _ if head.iter().find(|b| !b.is_ascii_whitespace()) == Some(&b'{') => Some(("json", MAX_JSON_BYTES)),
            _ => None,
        },
    }
}

pub async fn receive(mut mp: Multipart, kind: UploadKind, staging_dir: &Path) -> Result<Upload, ApiError> {
    let mut staged: Option<(String, PathBuf)> = None;
    let mut fields = HashMap::new();
    let result = async {
        while let Some(mut field) = mp.next_field().await.map_err(|e| ApiError::validation(e.to_string()))? {
            let name = field.name().unwrap_or_default().to_string();
            if field.file_name().is_none() {
                let mut text = Vec::new();
                while let Some(chunk) = field.chunk().await.map_err(|e| ApiError::validation(e.to_string()))? {
                    text.extend_from_slice(&chunk);
                    if text.len() > MAX_FIELD_BYTES {
                        return Err(too_large());
                    }
                }
                fields.insert(name, String::from_utf8_lossy(&text).into_owned());
                continue;
            }
            if staged.is_some() {
                return Err(ApiError::validation("only one file per upload"));
            }
            let id = Uuid::new_v4();
            let part = staging_dir.join(format!(".upload_{id}.part"));
            staged = Some((String::new(), part.clone()));
            let mut file = tokio::fs::File::create(&part)
                .await
                .map_err(|e| ApiError::from(targetscope_common::messages::ServiceError::io(e.to_string())))?;
            let mut size: u64 = 0;
            let mut limit: Option<(&str, u64)> = None;
            let mut head: Vec<u8> = Vec::new();
            while let Some(chunk) = field.chunk().await.map_err(|e| ApiError::validation(e.to_string()))? {
                size += chunk.len() as u64;
                if limit.is_none() {
                    head.extend_from_slice(&chunk[..chunk.len().min(16 - head.len().min(16))]);
                    if head.len() >= 8 || size >= 8 {
                        limit = Some(detect(kind, &head).ok_or_else(|| ApiError::validation("unexpected file type"))?);
                    }
                }
                if limit.is_some_and(|(_, max)| size > max) {
                    return Err(too_large());
                }
                file.write_all(&chunk).await.map_err(|e| ApiError::from(targetscope_common::messages::ServiceError::io(e.to_string())))?;
            }
            let (ext, _) = match limit {
                Some(l) => l,
                None => detect(kind, &head).ok_or_else(|| ApiError::validation("unexpected file type"))?,
            };
            file.flush().await.ok();
            let name = format!("upload_{id}.{ext}");
            tokio::fs::rename(&part, staging_dir.join(&name))
                .await
                .map_err(|e| ApiError::from(targetscope_common::messages::ServiceError::io(e.to_string())))?;
            staged = Some((name, staging_dir.join(format!("upload_{id}.{ext}"))));
        }
        Ok(())
    }
    .await;
    match (result, staged) {
        (Ok(()), Some((name, path))) if !name.is_empty() => Ok(Upload { staged_file: name, path, fields }),
        (Ok(()), _) => Err(ApiError::validation("no file in the upload")),
        (Err(e), staged) => {
            if let Some((_, path)) = staged {
                let _ = tokio::fs::remove_file(path).await;
            }
            Err(e)
        }
    }
}
