// doc/programs/web_server.md#rest-api
// doc/programs/web_server.md#uploads

use axum::extract::DefaultBodyLimit;
use axum::extract::{Multipart, State};
use axum::http::HeaderMap;
use axum::routing::post;
use axum::Router;
use targetscope_common::messages::importer::ImportFile;

use super::{call, id};
use crate::errors::{ApiError, ApiResult};
use crate::state::AppState;
use crate::uploads::{receive, UploadKind};

pub fn router() -> Router<AppState> {
    Router::new().route("/api/imports", post(import).layer(DefaultBodyLimit::disable()))
}

async fn import(State(st): State<AppState>, h: HeaderMap, mp: Multipart) -> ApiResult {
    let id = id(&h)?;
    let staging = st.folder("staging_dir").ok_or_else(|| ApiError::validation("staging_dir is not set"))?;
    let up = receive(mp, UploadKind::Import, &staging).await?;
    let result = call(&st, id, ImportFile { staged_file: up.staged_file.clone() }).await;
    if result.is_err() {
        let _ = tokio::fs::remove_file(&up.path).await;
    }
    result
}
