//! statistics

#[tokio::main]
async fn main() -> std::process::ExitCode {
    statistics::run().await
}
