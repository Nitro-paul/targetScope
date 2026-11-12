// doc/programs/web_server.md#rest-api
// doc/programs/web_server.md#camera-settings-menu

use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::routing::{get, post, put};
use axum::Router;
use serde::Deserialize;
use targetscope_common::messages::camera_reader::{CameraListControls, CameraListDevices, CameraSetControl, CameraStatus};
use targetscope_common::messages::image_processor::*;

use super::{call, id, parse};
use crate::admin::require;
use crate::errors::{ApiError, ApiResult};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/camera", get(status))
        .route("/api/camera/devices", get(devices))
        .route("/api/camera/controls", get(controls))
        .route("/api/camera/controls/{id}", put(set_control))
        .route("/api/camera/lens", get(lens_status).delete(lens_delete))
        .route("/api/camera/lens/{action}", post(lens_action))
        .route("/api/calibration", get(calibration))
}

async fn status(State(st): State<AppState>, h: HeaderMap) -> ApiResult {
    call(&st, id(&h)?, CameraStatus {}).await
}

async fn devices(State(st): State<AppState>, h: HeaderMap) -> ApiResult {
    call(&st, id(&h)?, CameraListDevices {}).await
}

async fn controls(State(st): State<AppState>, h: HeaderMap) -> ApiResult {
    call(&st, id(&h)?, CameraListControls {}).await
}

#[derive(Deserialize)]
struct Value {
    value: i64,
}

async fn set_control(State(st): State<AppState>, h: HeaderMap, Path(control): Path<i64>, b: Bytes) -> ApiResult {
    let v: Value = parse(&b)?;
    call(&st, id(&h)?, CameraSetControl { id: control, value: v.value }).await
}

async fn lens_status(State(st): State<AppState>, h: HeaderMap) -> ApiResult {
    call(&st, id(&h)?, LensCalibrationStatus {}).await
}

async fn lens_delete(State(st): State<AppState>, h: HeaderMap) -> ApiResult {
    require(&st, &h)?;
    call(&st, id(&h)?, LensCalibrationDelete {}).await
}

// doc/programs/web_server.md#admin-password
async fn lens_action(State(st): State<AppState>, h: HeaderMap, Path(action): Path<String>) -> ApiResult {
    let id = id(&h)?;
    match action.as_str() {
        "start" => {
            require(&st, &h)?;
            call(&st, id, LensCalibrationStart {}).await
        }
        "capture" => call(&st, id, LensCalibrationCapture {}).await,
        "finish" => {
            require(&st, &h)?;
            call(&st, id, LensCalibrationFinish {}).await
        }
        "cancel" => call(&st, id, LensCalibrationCancel {}).await,
        other => Err(ApiError::not_found(format!("/api/camera/lens/{other}"))),
    }
}

async fn calibration(State(st): State<AppState>, h: HeaderMap) -> ApiResult {
    call(&st, id(&h)?, LiveCalibrationStatus {}).await
}
