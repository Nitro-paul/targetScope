// doc/programs/web_server.md#rest-api

use axum::body::Bytes;
use axum::extract::{Multipart, Path, State};
use axum::http::HeaderMap;
use axum::routing::{get, put};
use axum::extract::DefaultBodyLimit;
use axum::Router;
use targetscope_common::messages::database_handler::{DeleteDevice, DeleteDeviceImage, ReadDevices, SaveDevice, SetDeviceImage};
use targetscope_common::model::Device;

use super::{call, id, parse};
use crate::admin::require;
use crate::errors::{ApiError, ApiResult};
use crate::state::AppState;
use crate::uploads::{receive, UploadKind};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/devices", get(list).post(create))
        .route("/api/devices/{id}", put(update).delete(delete))
        .route("/api/devices/{id}/image", put(set_image).delete(delete_image).layer(DefaultBodyLimit::disable()))
}

async fn list(State(st): State<AppState>, h: HeaderMap) -> ApiResult {
    call(&st, id(&h)?, ReadDevices::default()).await
}

async fn create(State(st): State<AppState>, h: HeaderMap, b: Bytes) -> ApiResult {
    let mut d: Device = parse(&b)?;
    d.id = None;
    call(&st, id(&h)?, SaveDevice(d)).await
}

async fn update(State(st): State<AppState>, h: HeaderMap, Path(device_id): Path<i64>, b: Bytes) -> ApiResult {
    let mut d: Device = parse(&b)?;
    d.id = Some(device_id);
    call(&st, id(&h)?, SaveDevice(d)).await
}

async fn delete(State(st): State<AppState>, h: HeaderMap, Path(device_id): Path<i64>) -> ApiResult {
    require(&st, &h)?;
    call(&st, id(&h)?, DeleteDevice { id: device_id }).await
}

// doc/programs/web_server.md#uploads
async fn set_image(State(st): State<AppState>, h: HeaderMap, Path(device_id): Path<i64>, mp: Multipart) -> ApiResult {
    let id = id(&h)?;
    let staging = st.folder("staging_dir").ok_or_else(|| ApiError::validation("staging_dir is not set"))?;
    let up = receive(mp, UploadKind::Image, &staging).await?;
    let result = call(&st, id, SetDeviceImage { device_id, staged_file: up.staged_file.clone() }).await;
    if result.is_err() {
        let _ = tokio::fs::remove_file(&up.path).await;
    }
    result
}

async fn delete_image(State(st): State<AppState>, h: HeaderMap, Path(device_id): Path<i64>) -> ApiResult {
    require(&st, &h)?;
    call(&st, id(&h)?, DeleteDeviceImage { device_id }).await
}
