//! `FrameFeeder`: publishes frames on `targetscope/frames/raw` or `targetscope/frames/live`
//! from images, videos or the synthetic renderer, at a chosen frame rate and resolution.

use std::path::PathBuf;

pub enum FrameSource {
    Image(PathBuf),
    Video(PathBuf),
    /// Name of a synthetic scene in testdata/synthetic/
    Synthetic(String),
}

pub struct FrameFeeder {
    // TODO: publisher thread
}

impl FrameFeeder {
    pub fn start(service: &str, source: FrameSource, fps: f64, width: u32, height: u32) -> Self {
        let _ = (service, source, fps, width, height);
        todo!()
    }

    pub fn stop(&mut self) {
        todo!()
    }
}
