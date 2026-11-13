// doc/programs/web_server.md#rest-api

use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::routing::{delete, get};
use axum::Router;
use targetscope_common::messages::exporter::{DeleteExport, ExportSessions, ListExports};

use super::{call, id, parse};
use crate::admin::require;
use crate::errors::ApiResult;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/api/exports", get(list).post(export)).route("/api/exports/{name}", delete(remove))
}

async fn list(State(st): State<AppState>, h: HeaderMap) -> ApiResult {
    call(&st, id(&h)?, ListExports {}).await
}

async fn export(State(st): State<AppState>, h: HeaderMap, b: Bytes) -> ApiResult {
    call(&st, id(&h)?, parse::<ExportSessions>(&b)?).await
}

async fn remove(State(st): State<AppState>, h: HeaderMap, Path(name): Path<String>) -> ApiResult {
    require(&st, &h)?;
    call(&st, id(&h)?, DeleteExport { name }).await
}
