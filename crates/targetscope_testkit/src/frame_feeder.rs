// doc/testing/overview.md#test-kit
// doc/frame_transport.md

use std::path::PathBuf;

pub enum FrameSource {
    Image(PathBuf),
    Video(PathBuf),
    Synthetic(String),
}

pub struct FrameFeeder;

impl FrameFeeder {
    pub fn start(_service: &str, _source: FrameSource, _fps: f64, _width: u32, _height: u32) -> Self {
        unimplemented!("doc/frame_transport.md")
    }
}
