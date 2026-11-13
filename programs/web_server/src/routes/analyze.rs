// doc/programs/web_server.md#rest-api

use axum::body::Bytes;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::routing::post;
use axum::Router;
use targetscope_common::messages::analyzer::AnalyzeShots;

use super::{call, id, parse};
use crate::errors::ApiResult;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/api/analyze", post(analyze))
}

async fn analyze(State(st): State<AppState>, h: HeaderMap, b: Bytes) -> ApiResult {
    call(&st, id(&h)?, parse::<AnalyzeShots>(&b)?).await
}
