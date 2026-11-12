// doc/programs/analyzer.md

#[tokio::main]
async fn main() -> std::process::ExitCode {
    analyzer::run().await
}
