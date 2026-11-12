// doc/programs/web_server.md#rest-api

use axum::body::Bytes;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::routing::post;
use axum::Router;
use targetscope_common::messages::statistics::ComputeStatistics;

use super::{call, id, parse};
use crate::errors::ApiResult;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/api/statistics", post(compute))
}

async fn compute(State(st): State<AppState>, h: HeaderMap, b: Bytes) -> ApiResult {
    call(&st, id(&h)?, parse::<ComputeStatistics>(&b)?).await
}
