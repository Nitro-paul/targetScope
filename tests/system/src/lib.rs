//! Harness of the system tests (doc/testing/system.md#setup): the launcher with a temporary
//! HOME and its default configuration (Redis on a free port), a camera through v4l2loopback fed
//! by ffmpeg (or mediamtx for RTSP), and a client for the REST API, /ws and /stream/*.mjpg.

pub mod camera;
pub mod client;
pub mod system;

pub use client::Client;
pub use system::System;
