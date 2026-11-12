//! web_server (doc/programs/web_server.md): serves the frontend in `static/`, the REST API
//! (each endpoint forwards to one Redis request), `/ws` with live events, MJPEG feeds and
//! read-only files from the image, staging and export folders.

pub mod admin;
pub mod errors;
pub mod files;
pub mod idempotency_key;
pub mod mjpeg;
pub mod routes;
pub mod state;
pub mod uploads;
pub mod ws;

use std::process::ExitCode;

/// Startup: logging, bus, settings cache, event readers for /ws, frame subscribers,
/// bind to the setting web_bind_address (default 0.0.0.0:8080), serve the router.
pub async fn run() -> ExitCode {
    todo!()
}
