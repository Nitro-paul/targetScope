// doc/programs/web_server.md#rest-api
// doc/programs/web_server.md#target-type-editor

use axum::body::Bytes;
use axum::extract::{Multipart, Path, State};
use axum::http::HeaderMap;
use axum::routing::{get, post, put};
use axum::extract::DefaultBodyLimit;
use axum::{Json, Router};
use serde_json::json;
use targetscope_common::messages::database_handler::{DeleteTargetType, ReadTargetTypes, SaveTargetType};
use targetscope_common::messages::image_processor::{CaptureRawImage, FindPaper, PrepareTargetPhoto};
use targetscope_common::model::TargetType;

use super::{call, id, parse, with_staged};
use crate::admin::require;
use crate::errors::{ApiError, ApiResult};
use crate::state::AppState;
use crate::uploads::{receive, UploadKind};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/target-types", get(list).post(create))
        .route("/api/target-types/{id}", put(update).delete(delete))
        .route("/api/target-types/photo", post(photo).layer(DefaultBodyLimit::disable()))
        .route("/api/target-types/photo/capture", post(capture))
        .route("/api/target-types/photo/prepare", post(prepare))
}

async fn list(State(st): State<AppState>, h: HeaderMap) -> ApiResult {
    call(&st, id(&h)?, ReadTargetTypes::default()).await
}

async fn create(State(st): State<AppState>, h: HeaderMap, b: Bytes) -> ApiResult {
    let mut t: TargetType = parse(&b)?;
    t.id = None;
    call(&st, id(&h)?, SaveTargetType(t)).await
}

async fn update(State(st): State<AppState>, h: HeaderMap, Path(type_id): Path<i64>, b: Bytes) -> ApiResult {
    let mut t: TargetType = parse(&b)?;
    t.id = Some(type_id);
    call(&st, id(&h)?, SaveTargetType(t)).await
}

async fn delete(State(st): State<AppState>, h: HeaderMap, Path(type_id): Path<i64>) -> ApiResult {
    require(&st, &h)?;
    call(&st, id(&h)?, DeleteTargetType { id: type_id }).await
}

// doc/interfaces.md#find_paper
async fn find(st: &AppState, staged_file: String) -> ApiResult {
    let r = st.bus.request(FindPaper { staged_file: staged_file.clone() }).await.map_err(|e| with_staged(e.into(), &staged_file))?;
    Ok(Json(json!({ "staged_file": staged_file, "corners": r.corners, "width_px": r.width_px, "height_px": r.height_px })))
}

async fn photo(State(st): State<AppState>, mp: Multipart) -> ApiResult {
    let staging = st.folder("staging_dir").ok_or_else(|| ApiError::validation("staging_dir is not set"))?;
    let up = receive(mp, UploadKind::Image, &staging).await?;
    find(&st, up.staged_file).await
}

async fn capture(State(st): State<AppState>) -> ApiResult {
    let r = st.bus.request(CaptureRawImage {}).await?;
    find(&st, r.staged_file).await
}

async fn prepare(State(st): State<AppState>, h: HeaderMap, b: Bytes) -> ApiResult {
    let req: PrepareTargetPhoto = parse(&b)?;
    let staged = req.staged_file.clone();
    call(&st, id(&h)?, req).await.map_err(|e| with_staged(e, &staged))
}
