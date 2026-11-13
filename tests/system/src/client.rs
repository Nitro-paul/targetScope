//! REST, WebSocket and MJPEG client, as the web UI would use them.

pub struct Client {
    pub base_url: String,
    // TODO: reqwest::Client with cookie store
}

impl Client {
    pub fn new(base_url: &str) -> Self {
        let _ = base_url;
        todo!()
    }

    pub async fn get(&self, path: &str) -> serde_json::Value {
        let _ = path;
        todo!()
    }

    pub async fn post(&self, path: &str, body: serde_json::Value) -> serde_json::Value {
        let _ = (path, body);
        todo!()
    }

    /// Opens /ws and returns a stream of event envelopes.
    pub async fn events(&self) -> tokio::sync::mpsc::Receiver<serde_json::Value> {
        todo!()
    }

    /// Frames per second measured on /stream/<name>.mjpg over the given time.
    pub async fn mjpeg_fps(&self, name: &str, over: std::time::Duration) -> f64 {
        let _ = (name, over);
        todo!()
    }
}
