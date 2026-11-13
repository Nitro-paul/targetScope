// doc/programs/web_server.md#rest-api

use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::routing::{get, put};
use axum::Router;
use targetscope_common::messages::database_handler::{DeleteShooter, ReadShooters, SaveShooter};
use targetscope_common::model::Shooter;

use super::{call, id, parse};
use crate::admin::require;
use crate::errors::ApiResult;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/api/shooters", get(list).post(create)).route("/api/shooters/{id}", put(update).delete(delete))
}

async fn list(State(st): State<AppState>, h: HeaderMap) -> ApiResult {
    call(&st, id(&h)?, ReadShooters::default()).await
}

async fn create(State(st): State<AppState>, h: HeaderMap, b: Bytes) -> ApiResult {
    let mut s: Shooter = parse(&b)?;
    s.id = None;
    call(&st, id(&h)?, SaveShooter(s)).await
}

async fn update(State(st): State<AppState>, h: HeaderMap, Path(shooter_id): Path<i64>, b: Bytes) -> ApiResult {
    let mut s: Shooter = parse(&b)?;
    s.id = Some(shooter_id);
    call(&st, id(&h)?, SaveShooter(s)).await
}

async fn delete(State(st): State<AppState>, h: HeaderMap, Path(shooter_id): Path<i64>) -> ApiResult {
    require(&st, &h)?;
    call(&st, id(&h)?, DeleteShooter { id: shooter_id }).await
}
