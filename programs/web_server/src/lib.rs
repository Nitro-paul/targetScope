// doc/programs/web_server.md

pub mod admin;
pub mod errors;
pub mod files;
pub mod idempotency_key;
pub mod mjpeg;
pub mod routes;
pub mod state;
pub mod uploads;
pub mod ws;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

use axum::extract::DefaultBodyLimit;
use axum::Router;
use targetscope_common::program;
use targetscope_common::settings::SettingsCache;
use tower_http::services::ServeDir;

use crate::state::{AppState, Inner};

pub const PROGRAM: &str = "web_server";
// doc/programs/web_server.md#rest-api
pub const JSON_BODY_LIMIT: usize = 4 * 1024 * 1024;

// doc/programs/web_server.md#frontend
pub fn static_dir() -> PathBuf {
    let installed = std::env::current_exe().ok().and_then(|e| e.parent().map(|d| d.join("static")));
    match installed.filter(|d| d.join("index.html").is_file()) {
        Some(dir) => dir,
        None => PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("static"),
    }
}

pub fn router(state: AppState) -> Router {
    let static_files = ServeDir::new(&state.static_dir).append_index_html_on_directories(true);
    Router::new()
        .merge(routes::api_router())
        .merge(files::router())
        .merge(mjpeg::router())
        .merge(ws::router())
        .fallback_service(static_files)
        .layer(DefaultBodyLimit::max(JSON_BODY_LIMIT))
        .with_state(state)
}

// doc/programs/web_server.md#access
pub async fn run() -> ExitCode {
    let p = program::start(PROGRAM).await;
    let settings = loop {
        match SettingsCache::load(&p.bus).await {
            Ok(s) => break s,
            Err(e) => {
                tracing::warn!(error = %e, "settings not available yet");
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }
    };
    let (events, _) = tokio::sync::broadcast::channel(ws::BUFFER);
    for source in ws::SOURCES {
        tokio::spawn(ws::read_events(p.bus.clone(), source, events.clone()));
    }
    let state = AppState {
        inner: Arc::new(Inner {
            bus: p.bus.clone(),
            settings: settings.clone(),
            admin: Default::default(),
            events,
            feeds: mjpeg::Feeds::new(),
            static_dir: static_dir(),
        }),
    };
    let address: SocketAddr = settings
        .get_parsed("web_bind_address")
        .unwrap_or_else(|| SocketAddr::from(([0, 0, 0, 0], 8080)));
    let listener = match tokio::net::TcpListener::bind(address).await {
        Ok(l) => l,
        Err(e) => {
            tracing::error!(error = %e, %address, "web_bind_address can't be used");
            return ExitCode::FAILURE;
        }
    };
    tracing::info!(%address, "web UI listening");
    let serving = p.bus.clone();
    tokio::spawn(async move {
        let _ = serving.serve(NoRequests).await;
    });
    let app = router(state).into_make_service_with_connect_info::<SocketAddr>();
    match axum::serve(listener, app).with_graceful_shutdown(program::shutdown_signal()).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            tracing::error!(error = %e, "web server ended");
            ExitCode::FAILURE
        }
    }
}

// doc/interfaces.md#requests-understood-by-all-programs
struct NoRequests;

#[async_trait::async_trait]
impl targetscope_common::bus::RequestHandler<serde_json::Value> for NoRequests {
    async fn handle(&self, _id: uuid::Uuid, _req: serde_json::Value) -> Result<serde_json::Value, targetscope_common::messages::ServiceError> {
        Err(targetscope_common::messages::ServiceError::new(
            targetscope_common::messages::ErrorCode::InvalidMessage,
            "web_server only answers ping",
        ))
    }
}
