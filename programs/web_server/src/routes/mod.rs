// doc/programs/web_server.md#rest-api

pub mod admin;
pub mod analyze;
pub mod camera;
pub mod devices;
pub mod exports;
pub mod imports;
pub mod saved_sessions;
pub mod session;
pub mod settings;
pub mod shooters;
pub mod statistics;
pub mod system;
pub mod target_types;

use axum::body::Bytes;
use axum::http::HeaderMap;
use axum::Json;
use axum::Router;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{json, Value};
use targetscope_common::messages::Request;
use uuid::Uuid;

use crate::errors::{ApiError, ApiResult};
use crate::idempotency_key;
use crate::state::AppState;

pub fn api_router() -> Router<AppState> {
    Router::new()
        .merge(session::router())
        .merge(saved_sessions::router())
        .merge(shooters::router())
        .merge(devices::router())
        .merge(target_types::router())
        .merge(camera::router())
        .merge(settings::router())
        .merge(statistics::router())
        .merge(analyze::router())
        .merge(exports::router())
        .merge(imports::router())
        .merge(system::router())
        .merge(admin::router())
}

// doc/programs/web_server.md#rest-api
pub async fn call<R: Request>(st: &AppState, id: Uuid, req: R) -> ApiResult
where
    R::Response: Serialize,
{
    let response = st.bus.request_with_id(id, req).await?;
    Ok(Json(serde_json::to_value(response).expect("serializable")))
}

pub fn id(headers: &HeaderMap) -> Result<Uuid, ApiError> {
    idempotency_key::request_id(headers)
}

pub fn parse<T: DeserializeOwned>(bytes: &Bytes) -> Result<T, ApiError> {
    let text: &[u8] = if bytes.iter().all(u8::is_ascii_whitespace) { b"{}" } else { bytes };
    serde_json::from_slice(text).map_err(|e| ApiError::validation(e.to_string()))
}

pub fn with_staged(e: ApiError, staged_file: &str) -> ApiError {
    let mut details = e.details.clone().unwrap_or_else(|| json!({}));
    if let Value::Object(map) = &mut details {
        map.insert("staged_file".into(), json!(staged_file));
    }
    e.with_details(details)
}
