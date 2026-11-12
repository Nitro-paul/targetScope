//! session_manager: one actor owns the current session; Bus::serve, the event readers and the
//! analysis task only forward inputs to it.

#[tokio::main]
async fn main() -> std::process::ExitCode {
    session_manager::run().await
}
