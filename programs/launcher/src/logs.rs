//! Reads stdout and stderr of every program line by line, prefixes them with the program
//! name and prints them to the launcher's stdout (doc/programs/launcher.md#logs).

pub fn forward_output(name: &str, child: &mut tokio::process::Child) {
    let _ = (name, child);
    todo!()
}
