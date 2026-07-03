use std::fs;
use std::path::PathBuf;
use std::process::Command;
use workspace_root::get_workspace_root;

fn build_dir() -> PathBuf {
    get_workspace_root().join("target/limine-tmp")
}

fn dest_dir() -> PathBuf {
    get_workspace_root().join("limine")
}

const LIMINE_URL: &str =
    "https://github.com/limine-bootloader/limine/releases/latest/download/limine-binary.tar.gz";

// Files to copy after running 'make'
const REQUIRED_FILES: &[&str] = &[
    "limine-bios.sys",
    "limine-bios-cd.bin",
    "limine-uefi-cd.bin",
    "BOOTX64.EFI",
    "BOOTIA32.EFI",
    "limine",
];

pub fn setup_limine() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Skip if already set up
    if dest_dir().exists() {
        return Ok(());
    }

    // 2. Download latest Limine source tarball
    let tar_path = get_workspace_root().join("target/limine.tar.gz");

    fs::create_dir_all(get_workspace_root().join("target"))?;
    let mut response = ureq::get(LIMINE_URL).call()?;
    let mut file = fs::File::create(&tar_path)?;
    std::io::copy(&mut response.body_mut().as_reader(), &mut file)?;

    // 3. Extract and Build
    println!("Extracting limine binary");
    fs::create_dir_all(build_dir())?;
    Command::new("tar")
        .args(["-xf"])
        .args([&tar_path])
        .args(["-C"])
        .args([&(build_dir())])
        .args(["--strip-components=1"])
        .status()?;

    println!("Compiling Limine...");
    Command::new("make").current_dir(build_dir()).status()?;

    // 4. Copy required files to /limine
    fs::create_dir_all(dest_dir())?;
    for file in REQUIRED_FILES {
        let src = build_dir().join(file);
        let dst = dest_dir().join(file);
        if src.exists() {
            fs::copy(src, dst)?;
        }
    }

    // 5. Cleanup
    fs::remove_dir_all(build_dir())?;
    fs::remove_file(tar_path)?;

    Ok(())
}
