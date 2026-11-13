// doc/programs/exporter.md

#[tokio::main]
async fn main() -> std::process::ExitCode {
    exporter::run().await
}
