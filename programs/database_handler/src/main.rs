// doc/programs/database_handler.md

#[tokio::main]
async fn main() -> std::process::ExitCode {
    database_handler::run().await
}
