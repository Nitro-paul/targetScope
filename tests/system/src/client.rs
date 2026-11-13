// doc/testing/system.md#setup

pub struct Client {
    pub base_url: String,
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

    pub async fn events(&self) -> tokio::sync::mpsc::Receiver<serde_json::Value> {
        todo!()
    }

    pub async fn mjpeg_fps(&self, name: &str, over: std::time::Duration) -> f64 {
        let _ = (name, over);
        todo!()
    }
}
