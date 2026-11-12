// doc/programs/web_server.md#rest-api

use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use targetscope_common::messages::database_handler::{DeleteSession, ReadSessions, SessionOrder};
use targetscope_common::model::SessionFilter;

use super::{call, id, parse};
use crate::admin::require;
use crate::errors::{ApiError, ApiResult};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/api/sessions/query", post(query)).route("/api/sessions/{id}", get(read).delete(delete))
}

#[derive(Deserialize)]
struct Query {
    #[serde(default)]
    filter: SessionFilter,
    order_by: Option<SessionOrder>,
    limit: Option<i64>,
    offset: Option<i64>,
}

async fn query(State(st): State<AppState>, h: HeaderMap, b: Bytes) -> ApiResult {
    let q: Query = parse(&b)?;
    call(&st, id(&h)?, ReadSessions { filter: q.filter, order_by: q.order_by, limit: q.limit, offset: q.offset, include_shots: false }).await
}

async fn read(State(st): State<AppState>, Path(session_id): Path<i64>) -> ApiResult {
    let filter = SessionFilter { session_ids: Some(vec![session_id]), ..Default::default() };
    let r = st.bus.request(ReadSessions { filter, order_by: None, limit: None, offset: None, include_shots: true }).await?;
    let s = r.sessions.into_iter().next().ok_or_else(|| ApiError::not_found(format!("session {session_id}")))?;
    Ok(Json(serde_json::to_value(s).expect("serializable")))
}

async fn delete(State(st): State<AppState>, h: HeaderMap, Path(session_id): Path<i64>) -> ApiResult {
    require(&st, &h)?;
    call(&st, id(&h)?, DeleteSession { id: session_id }).await
}
