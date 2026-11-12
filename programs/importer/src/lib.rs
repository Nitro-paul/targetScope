//! importer (doc/programs/importer.md): import_file. Every record is its own request; running
//! an interrupted import again is safe because records are found by their uuid.

pub mod archive;
pub mod consistency;
pub mod export_file;
pub mod handler;
pub mod records;

use std::process::ExitCode;

pub async fn run() -> ExitCode {
    todo!()
}
