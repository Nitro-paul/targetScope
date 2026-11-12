//! importer

#[tokio::main]
async fn main() -> std::process::ExitCode {
    importer::run().await
}
