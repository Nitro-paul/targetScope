//! analyzer

#[tokio::main]
async fn main() -> std::process::ExitCode {
    analyzer::run().await
}
