use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const BUILD_DIR: &str = "target/limine-tmp";
const DEST_DIR: &str = "limine";
const LIMINE_URL: &str = "https://github.com/limine-bootloader/limine/releases/latest/download/limine-binary.tar.gz";

// Files to copy after running 'make'
const REQUIRED_FILES: &[&str] = &[
    "limine-bios.sys",
    "limine-bios-cd.bin",
    "limine-uefi-cd.bin",
    "BOOTX64.EFI",
    "BOOTIA32.EFI",
    "limine"
];

pub fn setup_limine() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Skip if already set up
    if Path::new(DEST_DIR).exists() {
        return Ok(());
    }

    // 2. Download latest Limine source tarball
    let tar_path = Path::new("target/limine.tar.gz");
    
    fs::create_dir_all("target")?;
    let mut response = ureq::get(LIMINE_URL).call()?;
    let mut file = fs::File::create(tar_path)?;
    std::io::copy(&mut response.body_mut().as_reader(), &mut file)?;

    // 3. Extract and Build
    fs::create_dir_all(BUILD_DIR)?;
    Command::new("tar").args(["-xf", tar_path.to_str().unwrap(), "-C", BUILD_DIR, "--strip-components=1"]).status()?;

    println!("Compiling Limine...");
    Command::new("make").current_dir(BUILD_DIR).status()?;

    // 4. Copy required files to deps/limine
    fs::create_dir_all(DEST_DIR)?;
    for file in REQUIRED_FILES {
        let src = PathBuf::from(BUILD_DIR).join(file);
        let dst = PathBuf::from(DEST_DIR).join(file);
        if src.exists() {
            fs::copy(src, dst)?;
        }
    }

    // 5. Cleanup
    fs::remove_dir_all(BUILD_DIR)?;
    fs::remove_file(tar_path)?;
    
    Ok(())
}