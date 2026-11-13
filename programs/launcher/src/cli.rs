// doc/programs/launcher.md#usage

use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Run { config: Option<PathBuf> },
    ResetAdminPassword { config: Option<PathBuf> },
}

pub fn parse(args: &[String]) -> Result<Command, String> {
    let mut config = None;
    let mut reset = false;
    let mut it = args.iter().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--config" => config = Some(PathBuf::from(it.next().ok_or("--config needs a file")?)),
            "--reset-admin-password" => reset = true,
            other => return Err(format!("unknown argument {other}; usage: targetscope [--config <file>] [--reset-admin-password]")),
        }
    }
    Ok(if reset { Command::ResetAdminPassword { config } } else { Command::Run { config } })
}
