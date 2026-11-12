//! iceoryx2 frame services (doc/frame_transport.md).

pub const FRAMES_RAW: &str = "targetscope/frames/raw";
pub const FRAMES_LIVE: &str = "targetscope/frames/live";
/// Suffix of the matching event service, notified after each frame.
pub const NOTIFY_SUFFIX: &str = "/notify";

/// Service configuration, identical in every language (doc/frame_transport.md#iceoryx2-services).
pub const MAX_PUBLISHERS: usize = 1;
pub const MAX_SUBSCRIBERS: usize = 4;
pub const HISTORY_SIZE: usize = 0;
pub const SUBSCRIBER_MAX_BUFFER_SIZE: usize = 2;
pub const ENABLE_SAFE_OVERFLOW: bool = true;

/// Mirror of `schemas/frames/frame_header.h`.
/// TODO: #[derive(ZeroCopySend)] and #[type_name("TargetScopeFrameHeader")] (iceoryx2).
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct FrameHeader {
    pub seq: u64,
    pub timestamp_ns: u64,
    pub source_seq: u64,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub format: u32,
    pub px_per_mm: f64,
}

pub const FRAME_HEADER_TYPE_NAME: &str = "TargetScopeFrameHeader";

const _: () = assert!(std::mem::size_of::<FrameHeader>() == 48);
const _: () = assert!(std::mem::align_of::<FrameHeader>() == 8);
// TODO: field offset checks (std::mem::offset_of!)

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    Bgr8 = 1,
}

#[derive(Debug, thiserror::Error)]
pub enum FrameError {
    #[error("iceoryx2: {0}")]
    Iceoryx(String),
    #[error("timeout")]
    Timeout,
}

pub struct FramePublisher {
    // TODO: iceoryx2 publisher (slice of u8 with FrameHeader user header), notifier
}

pub struct FrameSubscriber {
    // TODO: iceoryx2 subscriber, listener
}

/// A received frame, borrowed from shared memory.
pub struct Frame<'a> {
    pub header: FrameHeader,
    pub data: &'a [u8],
}

// TODO: node type parameter (`&Node<ipc::Service>`)
pub fn frame_publisher(service: &str, max_len: usize) -> Result<FramePublisher, FrameError> {
    let _ = (service, max_len);
    todo!()
}

pub fn frame_subscriber(service: &str) -> Result<FrameSubscriber, FrameError> {
    let _ = service;
    todo!()
}

/// Removes resources of crashed iceoryx2 nodes.
pub fn cleanup_dead_nodes() -> Result<(), FrameError> {
    todo!()
}

impl FramePublisher {
    /// Loans a slice, lets the caller fill it, sends it and notifies the `/notify` service.
    pub fn publish(
        &mut self,
        header: FrameHeader,
        fill: impl FnOnce(&mut [u8]),
    ) -> Result<(), FrameError> {
        let _ = (header, fill);
        todo!()
    }
}

impl FrameSubscriber {
    /// Blocks on the event service and returns the newest sample.
    pub fn wait(&mut self, timeout: std::time::Duration) -> Result<Option<Frame<'_>>, FrameError> {
        let _ = timeout;
        todo!()
    }
}
