// doc/frame_transport.md#live-feed-to-the-browser-mjpeg

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::body::{Body, Bytes};
use axum::extract::State;
use axum::http::header;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use fast_image_resize::images::Image as ResizeImage;
use fast_image_resize::{PixelType, Resizer};
use targetscope_common::frames::{frame_subscriber, Frame, FrameSubscriber, FRAMES_LIVE, FRAMES_RAW};
use targetscope_common::settings::SettingsCache;
use tokio::sync::watch;

use crate::state::AppState;

pub const RAW_WIDTH: usize = 1280;
pub const FALLBACK_AFTER: Duration = Duration::from_secs(2);
const POLL: Duration = Duration::from_millis(100);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Live,
    Raw,
}

pub struct Feed {
    kind: Kind,
    jpeg: watch::Sender<Option<Bytes>>,
    viewers: AtomicUsize,
    running: AtomicBool,
}

pub struct Feeds {
    pub live: Arc<Feed>,
    pub raw: Arc<Feed>,
}

impl Feeds {
    pub fn new() -> Self {
        let feed = |kind| {
            Arc::new(Feed { kind, jpeg: watch::channel(None).0, viewers: AtomicUsize::new(0), running: AtomicBool::new(false) })
        };
        Feeds { live: feed(Kind::Live), raw: feed(Kind::Raw) }
    }
}

impl Default for Feeds {
    fn default() -> Self {
        Self::new()
    }
}

pub fn router() -> Router<AppState> {
    Router::new().route("/stream/live.mjpg", get(live)).route("/stream/raw.mjpg", get(raw))
}

async fn live(State(st): State<AppState>) -> Response {
    stream(st.feeds.live.clone(), st.settings.clone())
}

async fn raw(State(st): State<AppState>) -> Response {
    stream(st.feeds.raw.clone(), st.settings.clone())
}

struct Viewer(Arc<Feed>);

impl Drop for Viewer {
    fn drop(&mut self) {
        self.0.viewers.fetch_sub(1, Ordering::SeqCst);
    }
}

// doc/frame_transport.md#live-feed-to-the-browser-mjpeg
fn stream(feed: Arc<Feed>, settings: SettingsCache) -> Response {
    feed.viewers.fetch_add(1, Ordering::SeqCst);
    let viewer = Viewer(feed.clone());
    if !feed.running.swap(true, Ordering::SeqCst) {
        let (f, s) = (feed.clone(), settings);
        std::thread::Builder::new().name("mjpeg".into()).spawn(move || encode_loop(f, s)).expect("mjpeg thread");
    }
    let rx = feed.jpeg.subscribe();
    let body = futures::stream::unfold((rx, viewer), |(mut rx, viewer)| async move {
        loop {
            rx.changed().await.ok()?;
            let jpeg = rx.borrow_and_update().clone();
            if let Some(jpeg) = jpeg {
                let mut part = format!("--frame\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\n\r\n", jpeg.len()).into_bytes();
                part.extend_from_slice(&jpeg);
                part.extend_from_slice(b"\r\n");
                return Some((Ok::<_, std::io::Error>(Bytes::from(part)), (rx, viewer)));
            }
        }
    });
    (
        [(header::CONTENT_TYPE, "multipart/x-mixed-replace; boundary=frame"), (header::CACHE_CONTROL, "no-cache")],
        Body::from_stream(body),
    )
        .into_response()
}

fn subscribe(service: &str) -> Option<FrameSubscriber> {
    frame_subscriber(service).inspect_err(|e| tracing::warn!(error = %e, service, "frame subscriber failed")).ok()
}

// doc/frame_transport.md#live-feed-to-the-browser-mjpeg
fn encode_loop(feed: Arc<Feed>, settings: SettingsCache) {
    let mut live = if feed.kind == Kind::Live { subscribe(FRAMES_LIVE) } else { None };
    let mut raw: Option<FrameSubscriber> = if feed.kind == Kind::Raw { subscribe(FRAMES_RAW) } else { None };
    let mut last_live = Instant::now();
    let mut last_sent: Option<Instant> = None;
    while feed.viewers.load(Ordering::SeqCst) > 0 {
        let fps: u32 = settings.get_parsed("live_feed_fps").unwrap_or(15).clamp(1, 60);
        let quality: i32 = settings.get_parsed("live_feed_quality").unwrap_or(80).clamp(1, 100);
        let mut frame: Option<(Frame, bool)> = None;
        if let Some(sub) = live.as_mut() {
            if let Ok(Some(f)) = sub.wait(POLL) {
                last_live = Instant::now();
                frame = Some((f, false));
            }
        }
        if frame.is_none() && (feed.kind == Kind::Raw || last_live.elapsed() > FALLBACK_AFTER) {
            if raw.is_none() {
                raw = subscribe(FRAMES_RAW);
            }
            if let Some(Ok(Some(f))) = raw.as_mut().map(|s| s.wait(POLL)) {
                frame = Some((f, true));
            }
        } else if feed.kind == Kind::Live && raw.is_some() && last_live.elapsed() < FALLBACK_AFTER {
            raw = None;
        }
        let Some((frame, downscale)) = frame else {
            if live.is_none() && raw.is_none() {
                std::thread::sleep(POLL);
                live = if feed.kind == Kind::Live { subscribe(FRAMES_LIVE) } else { None };
            }
            continue;
        };
        if last_sent.is_some_and(|t| t.elapsed() < Duration::from_secs_f64(1.0 / fps as f64)) {
            continue;
        }
        match encode(&frame, downscale, quality) {
            Ok(jpeg) => {
                feed.jpeg.send_replace(Some(jpeg));
                last_sent = Some(Instant::now());
            }
            Err(e) => tracing::warn!(error = %e, "frame could not be encoded"),
        }
    }
    feed.running.store(false, Ordering::SeqCst);
}

// doc/frame_transport.md#live-feed-to-the-browser-mjpeg
pub fn encode(frame: &Frame, downscale: bool, quality: i32) -> Result<Bytes, String> {
    let h = frame.header();
    let (w, hh, stride) = (h.width as usize, h.height as usize, h.stride as usize);
    let data = frame.data();
    if data.len() < stride * hh || stride < w * 3 {
        return Err("frame smaller than its header".into());
    }
    let compress = |pixels: &[u8], width: usize, pitch: usize, height: usize| {
        let image = turbojpeg::Image { pixels, width, pitch, height, format: turbojpeg::PixelFormat::BGR };
        turbojpeg::compress(image, quality, turbojpeg::Subsamp::Sub2x2).map(|b| Bytes::copy_from_slice(&b)).map_err(|e| e.to_string())
    };
    if !downscale || w <= RAW_WIDTH {
        return compress(data, w, stride, hh);
    }
    let mut packed = Vec::with_capacity(w * hh * 3);
    for row in data.chunks(stride).take(hh) {
        packed.extend_from_slice(&row[..w * 3]);
    }
    let src = ResizeImage::from_vec_u8(w as u32, hh as u32, packed, PixelType::U8x3).map_err(|e| e.to_string())?;
    let (dw, dh) = (RAW_WIDTH, (hh * RAW_WIDTH / w).max(1));
    let mut dst = ResizeImage::new(dw as u32, dh as u32, PixelType::U8x3);
    Resizer::new().resize(&src, &mut dst, None).map_err(|e| e.to_string())?;
    compress(dst.buffer(), dw, dw * 3, dh)
}
