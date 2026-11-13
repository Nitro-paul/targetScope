// doc/testing/frame_transport.md#frame-header-contract

use std::mem::{align_of, offset_of, size_of};
use std::time::Duration;

use targetscope_common::frames::*;

fn service(name: &str) -> String {
    format!("targetscope/test/{name}/{}", std::process::id())
}

// FT-HDR-01
#[test]
fn ft_hdr_01_size_and_offsets() {
    assert_eq!((size_of::<FrameHeader>(), align_of::<FrameHeader>()), (48, 8));
    let offsets = [
        offset_of!(FrameHeader, seq),
        offset_of!(FrameHeader, timestamp_ns),
        offset_of!(FrameHeader, source_seq),
        offset_of!(FrameHeader, width),
        offset_of!(FrameHeader, height),
        offset_of!(FrameHeader, stride),
        offset_of!(FrameHeader, format),
        offset_of!(FrameHeader, px_per_mm),
    ];
    assert_eq!(offsets, [0, 8, 16, 24, 28, 32, 36, 40]);
}

// FT-HDR-02
#[test]
#[ignore = "needs the C++ test publisher (doc/testing/frame_transport.md#frame-header-contract)"]
fn ft_hdr_02_type_name() {}

// FT-HDR-03
#[test]
fn ft_hdr_03_wrong_header_type() {
    use iceoryx2::prelude::*;
    let name = service("wrong");
    let node = NodeBuilder::new().create::<ipc::Service>().unwrap();
    let sn: ServiceName = name.as_str().try_into().unwrap();
    let _other = node.service_builder(&sn).publish_subscribe::<[u8]>().user_header::<u64>().open_or_create().unwrap();
    assert!(frame_subscriber(&name).is_err());
}

// FT-HDR-04
#[test]
fn ft_hdr_04_pixel_format_value() {
    let name = service("format");
    let mut sub = frame_subscriber(&name).unwrap();
    let mut publ = frame_publisher(&name, 16 * 16 * 3).unwrap();
    let h = FrameHeader { seq: 1, width: 16, height: 16, stride: 48, format: PixelFormat::Bgr8 as u32, ..Default::default() };
    publ.publish(h, |_| {}).unwrap();
    assert_eq!(sub.wait(Duration::from_secs(1)).unwrap().unwrap().header().format, 1);
}
