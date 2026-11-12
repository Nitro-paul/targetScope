//! Files (doc/programs/web_server.md#files-and-streams): GET / (static/), /images/{devices|sessions}/{file},
//! /staging/{file} (only upload_*.jpg/png, raw_*, paper_*, photo_*), /exports/{file} (attachment).
//! Plain file names only, no "..".

use axum::Router;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    todo!()
}

pub fn is_plain_file_name(name: &str) -> bool {
    !name.is_empty() && !name.contains('/') && !name.contains('\\') && name != "." && name != ".."
}
