// doc/programs/importer.md

#[tokio::main]
async fn main() -> std::process::ExitCode {
    importer::run().await
}
