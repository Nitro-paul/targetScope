// doc/frame_transport.md

use std::time::Duration;

use iceoryx2::port::listener::Listener;
use iceoryx2::port::notifier::Notifier;
use iceoryx2::port::publisher::Publisher;
use iceoryx2::port::subscriber::Subscriber;
use iceoryx2::prelude::*;
use iceoryx2::sample::Sample;

pub const FRAMES_RAW: &str = "targetscope/frames/raw";
pub const FRAMES_LIVE: &str = "targetscope/frames/live";
pub const NOTIFY_SUFFIX: &str = "/notify";

// doc/frame_transport.md#iceoryx2-services
pub const MAX_PUBLISHERS: usize = 1;
pub const MAX_SUBSCRIBERS: usize = 4;
pub const HISTORY_SIZE: usize = 0;
pub const SUBSCRIBER_MAX_BUFFER_SIZE: usize = 2;
pub const ENABLE_SAFE_OVERFLOW: bool = true;

// doc/frame_transport.md#frame-format
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, ZeroCopySend)]
#[type_name("TargetScopeFrameHeader")]
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
const _: () = assert!(std::mem::offset_of!(FrameHeader, seq) == 0);
const _: () = assert!(std::mem::offset_of!(FrameHeader, timestamp_ns) == 8);
const _: () = assert!(std::mem::offset_of!(FrameHeader, source_seq) == 16);
const _: () = assert!(std::mem::offset_of!(FrameHeader, width) == 24);
const _: () = assert!(std::mem::offset_of!(FrameHeader, height) == 28);
const _: () = assert!(std::mem::offset_of!(FrameHeader, stride) == 32);
const _: () = assert!(std::mem::offset_of!(FrameHeader, format) == 36);
const _: () = assert!(std::mem::offset_of!(FrameHeader, px_per_mm) == 40);

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    Bgr8 = 1,
}

#[derive(Debug, thiserror::Error)]
pub enum FrameError {
    #[error("iceoryx2: {0}")]
    Iceoryx(String),
}

fn err<E: std::fmt::Debug>(e: E) -> FrameError {
    FrameError::Iceoryx(format!("{e:?}"))
}

type FrameService = iceoryx2::service::port_factory::publish_subscribe::PortFactory<ipc::Service, [u8], FrameHeader>;

fn node() -> Result<Node<ipc::Service>, FrameError> {
    NodeBuilder::new().create::<ipc::Service>().map_err(err)
}

// doc/frame_transport.md#iceoryx2-services
fn frame_service(node: &Node<ipc::Service>, service: &str) -> Result<FrameService, FrameError> {
    let name: ServiceName = service.try_into().map_err(err)?;
    node.service_builder(&name)
        .publish_subscribe::<[u8]>()
        .user_header::<FrameHeader>()
        .max_publishers(MAX_PUBLISHERS)
        .max_subscribers(MAX_SUBSCRIBERS)
        .history_size(HISTORY_SIZE)
        .subscriber_max_buffer_size(SUBSCRIBER_MAX_BUFFER_SIZE)
        .enable_safe_overflow(ENABLE_SAFE_OVERFLOW)
        .open_or_create()
        .map_err(err)
}

fn notify_service(
    node: &Node<ipc::Service>,
    service: &str,
) -> Result<iceoryx2::service::port_factory::event::PortFactory<ipc::Service>, FrameError> {
    let name: ServiceName = format!("{service}{NOTIFY_SUFFIX}").as_str().try_into().map_err(err)?;
    node.service_builder(&name).event().open_or_create().map_err(err)
}

pub struct FramePublisher {
    _node: Node<ipc::Service>,
    publisher: Publisher<ipc::Service, [u8], FrameHeader>,
    notifier: Notifier<ipc::Service>,
}

pub struct FrameSubscriber {
    _node: Node<ipc::Service>,
    subscriber: Subscriber<ipc::Service, [u8], FrameHeader>,
    listener: Listener<ipc::Service>,
}

pub struct Frame {
    sample: Sample<ipc::Service, [u8], FrameHeader>,
}

impl Frame {
    pub fn header(&self) -> &FrameHeader {
        self.sample.user_header()
    }

    pub fn data(&self) -> &[u8] {
        self.sample.payload()
    }
}

pub fn frame_publisher(service: &str, max_len: usize) -> Result<FramePublisher, FrameError> {
    let node = node()?;
    let publisher = frame_service(&node, service)?
        .publisher_builder()
        .initial_max_slice_len(max_len)
        .allocation_strategy(AllocationStrategy::PowerOfTwo)
        .create()
        .map_err(err)?;
    let notifier = notify_service(&node, service)?.notifier_builder().create().map_err(err)?;
    Ok(FramePublisher { _node: node, publisher, notifier })
}

pub fn frame_subscriber(service: &str) -> Result<FrameSubscriber, FrameError> {
    let node = node()?;
    let subscriber = frame_service(&node, service)?
        .subscriber_builder()
        .buffer_size(SUBSCRIBER_MAX_BUFFER_SIZE)
        .create()
        .map_err(err)?;
    let listener = notify_service(&node, service)?.listener_builder().create().map_err(err)?;
    Ok(FrameSubscriber { _node: node, subscriber, listener })
}

// doc/frame_transport.md#stale-shared-memory
pub fn cleanup_dead_nodes() -> Result<(), FrameError> {
    Node::<ipc::Service>::list(Config::global_config(), |state| {
        if let NodeState::<ipc::Service>::Dead(view) = state {
            if let Err(e) = view.try_remove_stale_resources() {
                tracing::warn!(error = ?e, "stale iceoryx2 resources could not be removed");
            }
        }
        CallbackProgression::Continue
    })
    .map_err(err)
}

impl FramePublisher {
    // doc/frame_transport.md#iceoryx2-services
    pub fn publish(&mut self, header: FrameHeader, fill: impl FnOnce(&mut [u8])) -> Result<(), FrameError> {
        let len = header.stride as usize * header.height as usize;
        let mut sample = self.publisher.loan_slice_uninit(len).map_err(err)?;
        *sample.user_header_mut() = header;
        let mut sample = sample.write_from_fn(|_| 0u8);
        fill(sample.payload_mut());
        sample.send().map_err(err)?;
        self.notifier.notify().map_err(err)?;
        Ok(())
    }
}

impl FrameSubscriber {
    // doc/frame_transport.md#subscriber-rules
    pub fn wait(&mut self, timeout: Duration) -> Result<Option<Frame>, FrameError> {
        if let Some(frame) = self.newest()? {
            return Ok(Some(frame));
        }
        self.listener.timed_wait(|_| {}, timeout).map_err(err)?;
        self.newest()
    }

    // doc/frame_transport.md#subscriber-rules
    fn newest(&mut self) -> Result<Option<Frame>, FrameError> {
        let mut newest = None;
        loop {
            match self.subscriber.receive() {
                Ok(Some(sample)) => newest = Some(Frame { sample }),
                Ok(None) => return Ok(newest),
                Err(e) if newest.is_some() => {
                    tracing::debug!(error = ?e, "receive stopped at the borrow limit");
                    return Ok(newest);
                }
                Err(e) => return Err(err(e)),
            }
        }
    }
}
