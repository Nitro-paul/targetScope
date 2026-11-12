//! database_handler: load config, init logging, open DB, start workers and maintenance.

#[tokio::main]
async fn main() -> std::process::ExitCode {
    database_handler::run().await
}
