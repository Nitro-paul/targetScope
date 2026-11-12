//! web_server

#[tokio::main]
async fn main() -> std::process::ExitCode {
    web_server::run().await
}
