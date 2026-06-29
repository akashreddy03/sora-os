use anyhow::{Result, anyhow};
use std::process::Command;

mod download_limine;

const REQUIRED_FILES: &[&str] = &[
    "limine/limine-bios.sys",
    "limine/limine-bios-cd.bin",
    "limine/limine-uefi-cd.bin",
    "limine/BOOTX64.EFI",
    "limine/BOOTIA32.EFI",
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::env::current_dir()?;
    let iso_dir = root.join("target/iso_root");
    let kernel_bin = root.join("target/x86_64-unknown-none/release/sora-os");

    run_command(
        "cargo",
        &[
            "build",
            "--release",
            "--manifest-path",
            "kernel/Cargo.toml",
            "--target",
            "x86_64-unknown-none",
        ],
    )?;

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
    
    let iso_path = root.join("sora.iso");
    run_command(
        "xorriso",
        &[
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
            iso_dir.to_str().unwrap(),
            "-o",
            iso_path.to_str().unwrap(),
        ],
    )?;

    run_command(
        "./limine/limine",
        &["bios-install", iso_path.to_str().unwrap()],
    )?;

    println!("ISO successfully created at: {:?}", iso_path);
    Ok(())
}

fn run_command(name: &str, args: &[&str]) -> Result<()> {
    let status = Command::new(name).args(args).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(anyhow!("Command {} failed with status {}", name, status))
    }
}
