// doc/programs/web_server.md

#[tokio::main]
async fn main() -> std::process::ExitCode {
    web_server::run().await
}
