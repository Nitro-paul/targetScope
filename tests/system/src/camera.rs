// doc/testing/system.md#setup

pub struct LoopbackCamera {
    pub device: String,
}

impl LoopbackCamera {
    pub fn start(video: &str) -> Self {
        let _ = video;
        todo!()
    }

    pub fn pause(&mut self) {
        todo!()
    }

    pub fn resume(&mut self) {
        todo!()
    }
}

pub struct RtspCamera {
    pub url: String,
}

impl RtspCamera {
    pub fn start(video: &str) -> Self {
        let _ = video;
        todo!()
    }
}
