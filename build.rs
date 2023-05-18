use std::{os::unix::process::CommandExt, process::Command};

fn main() {
    Command::new("yarn").arg("build").exec();
}
