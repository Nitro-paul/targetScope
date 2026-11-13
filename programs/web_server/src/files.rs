// doc/programs/web_server.md#files-and-streams

use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;

use crate::errors::ApiError;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/images/{sub}/{file}", get(image)).route("/staging/{file}", get(staged)).route("/exports/{file}", get(export))
}

pub fn is_plain_file_name(name: &str) -> bool {
    !name.is_empty() && !name.contains('/') && !name.contains('\\') && name != "." && name != ".." && !name.contains('\0')
}

fn content_type(name: &str) -> &'static str {
    match name.rsplit('.').next().unwrap_or("") {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "csv" => "text/csv; charset=utf-8",
        "json" => "application/json",
        "zip" => "application/zip",
        _ => "application/octet-stream",
    }
}

async fn serve(path: std::path::PathBuf, name: &str, attachment: bool) -> Result<Response, ApiError> {
    let file = tokio::fs::File::open(&path).await.map_err(|_| ApiError::not_found(name.to_string()))?;
    let mut r = Body::from_stream(tokio_util_stream(file)).into_response();
    r.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type(name)));
    if attachment {
        if let Ok(v) = HeaderValue::from_str(&format!("attachment; filename=\"{name}\"")) {
            r.headers_mut().insert(header::CONTENT_DISPOSITION, v);
        }
    }
    Ok(r)
}

fn tokio_util_stream(file: tokio::fs::File) -> impl futures::Stream<Item = std::io::Result<Vec<u8>>> {
    futures::stream::unfold(file, |mut f| async move {
        use tokio::io::AsyncReadExt;
        let mut buf = vec![0u8; 64 * 1024];
        match f.read(&mut buf).await {
            Ok(0) => None,
            Ok(n) => {
                buf.truncate(n);
                Some((Ok(buf), f))
            }
            Err(e) => Some((Err(e), f)),
        }
    })
}

async fn image(State(st): State<AppState>, Path((sub, file)): Path<(String, String)>) -> Result<Response, ApiError> {
    if !matches!(sub.as_str(), "devices" | "sessions") || !is_plain_file_name(&file) {
        return Err(ApiError::not_found(file));
    }
    let dir = st.folder("image_dir").ok_or_else(|| ApiError::not_found("image folder"))?;
    serve(dir.join(&sub).join(&file), &file, false).await
}

// doc/programs/web_server.md#files-and-streams
pub fn staged_visible(name: &str) -> bool {
    let image = name.ends_with(".jpg") || name.ends_with(".png");
    (name.starts_with("upload_") && image) || name.starts_with("raw_") || name.starts_with("paper_") || name.starts_with("photo_")
}

async fn staged(State(st): State<AppState>, Path(file): Path<String>) -> Result<Response, ApiError> {
    if !is_plain_file_name(&file) || !staged_visible(&file) {
        return Err(ApiError::not_found(file));
    }
    let dir = st.folder("staging_dir").ok_or_else(|| ApiError::not_found("staging folder"))?;
    serve(dir.join(&file), &file, false).await
}

async fn export(State(st): State<AppState>, Path(file): Path<String>) -> Result<Response, ApiError> {
    if !is_plain_file_name(&file) || file.starts_with('.') {
        return Err(ApiError::not_found(file));
    }
    let dir = st.folder("export_dir").ok_or_else(|| ApiError::new(StatusCode::CONFLICT, "setting_not_set", "export_dir is not set"))?;
    serve(dir.join(&file), &file, true).await
}
