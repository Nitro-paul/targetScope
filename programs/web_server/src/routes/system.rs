// doc/programs/web_server.md#rest-api

use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::routing::{get, post};
use axum::Router;
use targetscope_common::messages::launcher::{LauncherStatus, RestartProgram};

use super::{call, id};
use crate::admin::require;
use crate::errors::ApiResult;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/api/system", get(status)).route("/api/system/programs/{name}/restart", post(restart))
}

async fn status(State(st): State<AppState>, h: HeaderMap) -> ApiResult {
    call(&st, id(&h)?, LauncherStatus {}).await
}

async fn restart(State(st): State<AppState>, h: HeaderMap, Path(name): Path<String>) -> ApiResult {
    require(&st, &h)?;
    call(&st, id(&h)?, RestartProgram { name }).await
}
