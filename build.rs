use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let kernel = PathBuf::from(std::env::var_os("CARGO_BIN_FILE_KERNEL_kernel").unwrap());

    let bios_path = out_dir.join("bios.img");
    bootloader::BiosBoot::new(&kernel)
        .create_disk_image(&bios_path)
        .unwrap();

    let uefi_path = out_dir.join("uefi.img");
    bootloader::UefiBoot::new(&kernel)
        .create_disk_image(&uefi_path)
        .unwrap();

    let stable_dir = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap())
        .join("target")
        .join("flynn_os");
    fs::create_dir_all(&stable_dir).unwrap();
    fs::copy(&bios_path, stable_dir.join("bios.img")).unwrap();
    fs::copy(&uefi_path, stable_dir.join("uefi.img")).unwrap();

    println!("cargo:rustc-env=BIOS_PATH={}", bios_path.display());
    println!("cargo:rustc-env=UEFI_PATH={}", uefi_path.display());
    println!(
        "cargo:warning=Boot images copied to {}",
        stable_dir.display()
    );
}
