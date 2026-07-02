use std::path::PathBuf;
use crate::build::{build};
use std::process::Command;
use workspace_root::get_workspace_root;

pub fn run(kernel_bin: Option<&String>) -> Result<(), Box<dyn std::error::Error>> {
    build(Some(PathBuf::from(kernel_bin.unwrap())))?;
    Command::new("qemu-system-x86_64").args(["-cdrom"]).args([get_workspace_root().join("sora.iso")]).args(["--serial", "stdio"]).status()?;
    Ok(())
}