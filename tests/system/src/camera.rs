//! Test cameras: a v4l2loopback device fed by ffmpeg with videos from testdata/videos/,
//! or mediamtx streaming a video over RTSP.

pub struct LoopbackCamera {
    pub device: String,
    // TODO: ffmpeg child
}

impl LoopbackCamera {
    pub fn start(video: &str) -> Self {
        let _ = video;
        todo!()
    }

    /// Stops feeding frames (camera gone) until resume().
    pub fn pause(&mut self) {
        todo!()
    }

    pub fn resume(&mut self) {
        todo!()
    }
}

pub struct RtspCamera {
    pub url: String,
    // TODO: mediamtx and ffmpeg children
}

impl RtspCamera {
    pub fn start(video: &str) -> Self {
        let _ = video;
        todo!()
    }
}
