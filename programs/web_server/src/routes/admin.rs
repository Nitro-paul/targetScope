// doc/programs/web_server.md#admin-password

use std::net::SocketAddr;

use axum::body::Bytes;
use axum::extract::{ConnectInfo, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;
use targetscope_common::messages::database_handler::SaveSetting;
use targetscope_common::model::Setting;

use super::parse;
use crate::admin::{clear_cookie, hash_password, password_hash, set_cookie, verify_password, SETTING};
use crate::errors::{ApiError, ApiResult};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/admin", get(status))
        .route("/api/admin/login", post(login))
        .route("/api/admin/logout", post(logout))
        .route("/api/admin/password", put(password))
}

async fn status(State(st): State<AppState>, h: HeaderMap) -> ApiResult {
    Ok(Json(json!({ "protected": password_hash(&st).is_some(), "logged_in": st.admin.logged_in(&h) })))
}

#[derive(Deserialize)]
struct Login {
    password: String,
}

fn wrong() -> ApiError {
    ApiError::new(StatusCode::UNAUTHORIZED, "wrong_password", "wrong password")
}

async fn login(State(st): State<AppState>, ConnectInfo(addr): ConnectInfo<SocketAddr>, b: Bytes) -> Result<Response, ApiError> {
    st.admin.check_attempt(addr.ip())?;
    let l: Login = parse(&b)?;
    let Some(hash) = password_hash(&st) else {
        return Ok(Json(json!({ "protected": false, "logged_in": false })).into_response());
    };
    if !verify_password(&l.password, &hash) {
        st.admin.wrong_attempt(addr.ip());
        return Err(wrong());
    }
    let token = st.admin.login();
    Ok(([(header::SET_COOKIE, set_cookie(&token))], Json(json!({ "protected": true, "logged_in": true }))).into_response())
}

async fn logout(State(st): State<AppState>, h: HeaderMap) -> Response {
    st.admin.logout(&h);
    ([(header::SET_COOKIE, clear_cookie())], Json(json!({ "logged_in": false }))).into_response()
}

#[derive(Deserialize)]
struct Change {
    current_password: Option<String>,
    new_password: Option<String>,
}

// doc/programs/web_server.md#admin-password
async fn password(State(st): State<AppState>, ConnectInfo(addr): ConnectInfo<SocketAddr>, b: Bytes) -> ApiResult {
    let c: Change = parse(&b)?;
    if let Some(hash) = password_hash(&st) {
        st.admin.check_attempt(addr.ip())?;
        if !c.current_password.as_deref().is_some_and(|p| verify_password(p, &hash)) {
            st.admin.wrong_attempt(addr.ip());
            return Err(wrong());
        }
    }
    let (active, value) = match c.new_password.filter(|p| !p.is_empty()) {
        Some(p) => (true, hash_password(&p)?),
        None => (false, String::new()),
    };
    let setting = Setting {
        name: SETTING.into(),
        display_name: st.settings.setting(SETTING).map(|s| s.display_name).unwrap_or_else(|| "Admin password".into()),
        active,
        value,
        optional: true,
        default_value: String::new(),
        default_active: false,
        value_rule: String::new(),
    };
    st.bus.request(SaveSetting(setting)).await?;
    st.admin.logout_all();
    Ok(Json(json!({ "protected": active })))
}
