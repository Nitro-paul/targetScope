// doc/programs/launcher.md#usage

#[tokio::main]
async fn main() -> std::process::ExitCode {
    launcher::run(std::env::args().collect()).await
}
