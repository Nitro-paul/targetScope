// doc/programs/session_manager.md

#[tokio::main]
async fn main() -> std::process::ExitCode {
    session_manager::run().await
}
