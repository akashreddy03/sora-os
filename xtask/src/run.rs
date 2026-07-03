use crate::build::build;
use std::path::PathBuf;
use std::process::Command;
use workspace_root::get_workspace_root;

pub fn run(bin: Option<&String>) -> Result<(), Box<dyn std::error::Error>> {
    let kernel_bin = bin.unwrap();
    build(Some(PathBuf::from(kernel_bin)))?;
    let mut run_cmd = Command::new("qemu-system-x86_64");
    run_cmd
        .arg("-cdrom")
        .arg(get_workspace_root().join("sora.iso"))
        .args([
            "--serial",
            "stdio",
            "-device",
            "isa-debug-exit,iobase=0xf4,iosize=0x04",
        ]);
    if kernel_bin.contains("/deps/") {
        run_cmd.args(["-display", "none"]);
    }
    let status = run_cmd.status()?;
    let code = status.code().unwrap_or(1);
    match code {
        35 => std::process::exit(101),
        33 => std::process::exit(0),
        _ => std::process::exit(code),
    }
}
