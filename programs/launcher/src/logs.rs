// doc/programs/launcher.md#logs

use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::process::Child;

pub fn forward_output(name: &str, child: &mut Child) {
    if let Some(out) = child.stdout.take() {
        tokio::spawn(forward(name.to_string(), out));
    }
    if let Some(err) = child.stderr.take() {
        tokio::spawn(forward(name.to_string(), err));
    }
}

async fn forward<R: AsyncRead + Unpin>(name: String, stream: R) {
    let mut lines = BufReader::new(stream).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        println!("[{name}] {line}");
    }
}
