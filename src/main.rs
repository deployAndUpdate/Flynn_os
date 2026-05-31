use std::env;
use std::path::PathBuf;
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let bios_path = env!("BIOS_PATH");
    let uefi_path = env!("UEFI_PATH");

    let args: Vec<String> = env::args().collect();
    let prog = args.first().map(String::as_str).unwrap_or("flynn_os");

    let mode = match args.get(1).map(String::as_str) {
        Some("bios") => BootMode::Bios,
        Some("uefi") => BootMode::Uefi,
        Some("-h") | Some("--help") => {
            print_help(prog);
            return ExitCode::SUCCESS;
        }
        Some(other) => {
            eprintln!("Unknown mode: {other}");
            print_help(prog);
            return ExitCode::from(2);
        }
        None => BootMode::Bios,
    };

    let mut cmd = Command::new("qemu-system-x86_64");
    cmd.arg("-m").arg("512M");
    cmd.arg("-serial").arg("stdio");

    match mode {
        BootMode::Bios => {
            cmd.arg("-drive").arg(format!("format=raw,file={bios_path}"));
        }
        BootMode::Uefi => {
            let ovmf_code = ovmf_path("FLYNN_OVMF_CODE", "/usr/share/OVMF/OVMF_CODE.fd");
            let ovmf_vars = ovmf_path("FLYNN_OVMF_VARS", "/usr/share/OVMF/OVMF_VARS.fd");

            cmd.arg("-drive").arg(format!("format=raw,file={uefi_path}"));
            cmd.arg("-drive").arg(format!(
                "if=pflash,format=raw,unit=0,file={},readonly=on",
                ovmf_code.display()
            ));
            cmd.arg("-drive").arg(format!(
                "if=pflash,format=raw,unit=1,file={}",
                ovmf_vars.display()
            ));
        }
    }

    let status = cmd.status().expect("failed to start qemu-system-x86_64");
    match status.code() {
        Some(0) => ExitCode::SUCCESS,
        Some(code) => ExitCode::from(code as u8),
        None => ExitCode::FAILURE,
    }
}

enum BootMode {
    Bios,
    Uefi,
}

fn ovmf_path(env_var: &str, default: &str) -> PathBuf {
    env::var_os(env_var)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(default))
}

fn print_help(prog: &str) {
    eprintln!("Usage: {prog} [bios|uefi]");
    eprintln!();
    eprintln!("  bios   Boot legacy BIOS image (default)");
    eprintln!("  uefi   Boot UEFI image with OVMF");
    eprintln!();
    eprintln!("Stable disk images after build:");
    eprintln!("  target/flynn_os/bios.img");
    eprintln!("  target/flynn_os/uefi.img");
    eprintln!();
    eprintln!("Manual QEMU (BIOS):");
    eprintln!("  qemu-system-x86_64 -m 512M -drive format=raw,file=target/flynn_os/bios.img");
    eprintln!();
    eprintln!("UEFI firmware paths (override with env vars):");
    eprintln!("  FLYNN_OVMF_CODE  (default: /usr/share/OVMF/OVMF_CODE.fd)");
    eprintln!("  FLYNN_OVMF_VARS  (default: /usr/share/OVMF/OVMF_VARS.fd)");
}
