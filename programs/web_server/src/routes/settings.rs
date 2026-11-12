// doc/programs/web_server.md#rest-api
// doc/programs/web_server.md#settings-view

use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde::Deserialize;
use targetscope_common::messages::database_handler::{ChangeImageDir, ReadSettings, ReadSettingsResponse, ResetSettings, SaveSetting};
use targetscope_common::model::Setting;

use super::{call, id, parse};
use crate::admin::{require, SETTING};
use crate::errors::{ApiError, ApiResult};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/settings", get(list))
        .route("/api/settings/reset", post(reset))
        .route("/api/settings/image_dir", put(image_dir))
        .route("/api/settings/{name}", put(save))
}

async fn list(State(st): State<AppState>) -> ApiResult {
    let mut r: ReadSettingsResponse = st.bus.request(ReadSettings::default()).await?;
    r.settings.retain(|s| s.name != SETTING);
    Ok(Json(serde_json::to_value(r).expect("serializable")))
}

#[derive(Deserialize)]
struct Value {
    value: String,
    active: bool,
}

async fn save(State(st): State<AppState>, h: HeaderMap, Path(name): Path<String>, b: Bytes) -> ApiResult {
    if name == SETTING {
        return Err(ApiError::validation("the admin password is set with /api/admin/password"));
    }
    require(&st, &h)?;
    let v: Value = parse(&b)?;
    let display_name = st.settings.setting(&name).map(|s| s.display_name).unwrap_or_else(|| name.clone());
    let setting = Setting {
        name,
        display_name,
        active: v.active,
        value: v.value,
        optional: false,
        default_value: String::new(),
        default_active: true,
        value_rule: String::new(),
    };
    call(&st, id(&h)?, SaveSetting(setting)).await
}

async fn reset(State(st): State<AppState>, h: HeaderMap, b: Bytes) -> ApiResult {
    require(&st, &h)?;
    call(&st, id(&h)?, parse::<ResetSettings>(&b)?).await
}

async fn image_dir(State(st): State<AppState>, h: HeaderMap, b: Bytes) -> ApiResult {
    require(&st, &h)?;
    call(&st, id(&h)?, parse::<ChangeImageDir>(&b)?).await
}
