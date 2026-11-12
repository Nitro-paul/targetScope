// doc/programs/web_server.md#rest-api
// doc/interfaces.md#session_manager

use axum::body::Bytes;
use axum::extract::{FromRequest, Multipart, Path, Request, State};
use axum::http::{header, HeaderMap};
use axum::routing::{get, post, put};
use axum::extract::DefaultBodyLimit;
use axum::Router;
use targetscope_common::messages::session_manager::*;
use targetscope_common::model::Corners;

use super::{call, id, parse, with_staged};
use crate::errors::{ApiError, ApiResult};
use crate::state::AppState;
use crate::uploads::{receive, UploadKind};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/session", get(get_session))
        .route("/api/session/setup", post(setup).patch(update_setup))
        .route("/api/session/notes", put(notes))
        .route("/api/session/new-paper", post(new_paper))
        .route("/api/session/calibrate", post(calibrate))
        .route("/api/session/{action}", post(action))
        .route("/api/session/shots", post(add_shot))
        .route("/api/session/shots/{index}", put(update_shot).delete(delete_shot))
        .route("/api/session/photo", post(photo).layer(DefaultBodyLimit::disable()))
}

async fn get_session(State(st): State<AppState>, h: HeaderMap) -> ApiResult {
    call(&st, id(&h)?, SessionGet {}).await
}

async fn setup(State(st): State<AppState>, h: HeaderMap, b: Bytes) -> ApiResult {
    call(&st, id(&h)?, parse::<SessionSetupRequest>(&b)?).await
}

async fn update_setup(State(st): State<AppState>, h: HeaderMap, b: Bytes) -> ApiResult {
    call(&st, id(&h)?, parse::<SessionUpdateSetup>(&b)?).await
}

async fn notes(State(st): State<AppState>, h: HeaderMap, b: Bytes) -> ApiResult {
    call(&st, id(&h)?, parse::<SessionSetNotes>(&b)?).await
}

async fn new_paper(State(st): State<AppState>, h: HeaderMap) -> ApiResult {
    call(&st, id(&h)?, SessionNewPaper {}).await
}

async fn calibrate(State(st): State<AppState>, h: HeaderMap, b: Bytes) -> ApiResult {
    let corners: Corners = parse(&b)?;
    call(&st, id(&h)?, SessionCalibrate { corners }).await
}

async fn action(State(st): State<AppState>, h: HeaderMap, Path(action): Path<String>) -> ApiResult {
    let id = id(&h)?;
    match action.as_str() {
        "start" => call(&st, id, SessionStart {}).await,
        "pause" => call(&st, id, SessionPause {}).await,
        "resume" => call(&st, id, SessionResume {}).await,
        "stop" => call(&st, id, SessionStop {}).await,
        "save" => call(&st, id, SessionSave {}).await,
        "cancel" => call(&st, id, SessionCancel {}).await,
        other => Err(ApiError::not_found(format!("/api/session/{other}"))),
    }
}

async fn add_shot(State(st): State<AppState>, h: HeaderMap, b: Bytes) -> ApiResult {
    call(&st, id(&h)?, parse::<SessionAddShot>(&b)?).await
}

#[derive(serde::Deserialize)]
struct Position {
    angle_turns: f64,
    radius_mm: f64,
}

async fn update_shot(State(st): State<AppState>, h: HeaderMap, Path(index): Path<i64>, b: Bytes) -> ApiResult {
    let p: Position = parse(&b)?;
    call(&st, id(&h)?, SessionUpdateShot { index, angle_turns: p.angle_turns, radius_mm: p.radius_mm }).await
}

async fn delete_shot(State(st): State<AppState>, h: HeaderMap, Path(index): Path<i64>) -> ApiResult {
    call(&st, id(&h)?, SessionDeleteShot { index }).await
}

// doc/programs/web_server.md#uploads
async fn photo(State(st): State<AppState>, req: Request) -> ApiResult {
    let h = req.headers().clone();
    let id = id(&h)?;
    let multipart = h.get(header::CONTENT_TYPE).and_then(|v| v.to_str().ok()).is_some_and(|v| v.starts_with("multipart/"));
    if !multipart {
        let b = Bytes::from_request(req, &st).await.map_err(|e| ApiError::validation(e.to_string()))?;
        return call(&st, id, parse::<SessionUploadPhoto>(&b)?).await;
    }
    let mp = Multipart::from_request(req, &st).await.map_err(|e| ApiError::validation(e.to_string()))?;
    let staging = st.folder("staging_dir").ok_or_else(|| ApiError::validation("staging_dir is not set"))?;
    let up = receive(mp, UploadKind::Image, &staging).await?;
    let corners = match up.fields.get("corners").filter(|c| !c.trim().is_empty() && c.trim() != "null") {
        Some(c) => Some(serde_json::from_str::<Corners>(c).map_err(|e| ApiError::validation(e.to_string()))?),
        None => None,
    };
    let result = call(&st, id, SessionUploadPhoto { staged_file: up.staged_file.clone(), corners }).await;
    match result {
        Err(e) if e.code == "processing_failed" => Err(with_staged(e, &up.staged_file)),
        Err(e) => {
            let _ = tokio::fs::remove_file(&up.path).await;
            Err(e)
        }
        ok => ok,
    }
}
