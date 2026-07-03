use crate::download_limine;
use anyhow::Result;
use std::{
    path::PathBuf,
    process::{Command, Stdio},
};
use workspace_root::get_workspace_root;

const REQUIRED_FILES: &[&str] = &[
    "limine/limine-bios.sys",
    "limine/limine-bios-cd.bin",
    "limine/limine-uefi-cd.bin",
    "limine/BOOTX64.EFI",
    "limine/BOOTIA32.EFI",
];

pub fn build(kernel_bin: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    let root = get_workspace_root();
    let iso_dir = root.join("target/iso_root");

    let skip_build = kernel_bin.is_some();

    let kernel_bin = kernel_bin.unwrap_or(root.join("target/x86_64-unknown-none/release/sora-os"));
    println!("BUILD SCRIPT STARTED!!!");

    if !skip_build {
        Command::new("cargo")
            .args([
                "build",
                "--release",
                "--manifest-path",
                "kernel/Cargo.toml",
                "--target",
                "x86_64-unknown-none",
            ])
            .status()?;
    } else {
        println!("Skipping build as binary is already provided.");
    }

    download_limine::setup_limine()?;

    if iso_dir.exists() {
        std::fs::remove_dir_all(&iso_dir)?;
    }
    std::fs::create_dir_all(iso_dir.join("boot/limine"))?;
    std::fs::create_dir_all(iso_dir.join("EFI/BOOT"))?;

    std::fs::copy(&kernel_bin, iso_dir.join("boot/sora.elf"))?;
    std::fs::copy(root.join("limine.conf"), iso_dir.join("boot/limine.conf"))?;

    for file in REQUIRED_FILES {
        let src = root.join(file);
        let dst = iso_dir.join("boot").join(file);
        if src.exists() {
            std::fs::copy(src, dst)?;
        }
    }
    println!("Building ISO...");
    let iso_path = root.join("sora.iso");
    Command::new("xorriso")
        .args([
            "-as",
            "mkisofs",
            "-b",
            "boot/limine/limine-bios-cd.bin",
            "-no-emul-boot",
            "-boot-load-size",
            "4",
            "-boot-info-table",
            "--efi-boot",
            "/boot/limine/limine-uefi-cd.bin",
            "-efi-boot-part",
            "--efi-boot-image",
            "--protective-msdos-label",
        ])
        .arg(iso_dir)
        .arg("-o")
        .arg(&iso_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;

    println!("Installing limine bios stages on kernel iso");

    Command::new(root.join("limine/limine"))
        .arg("bios-install")
        .arg(&iso_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;

    println!("ISO successfully created at: {:?}", iso_path);
    Ok(())
}
