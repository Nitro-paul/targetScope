//! MJPEG feeds (doc/frame_transport.md#live-feed-to-the-browser-mjpeg): /stream/live.mjpg from
//! targetscope/frames/live, /stream/raw.mjpg from targetscope/frames/raw; frames are scaled
//! (fast_image_resize), JPEG encoded once (turbojpeg) at live_feed_fps and shared by all clients.

use axum::Router;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    todo!()
}
