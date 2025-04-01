use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

use clap::Parser;
use winreg::{enums::HKEY_CLASSES_ROOT, RegKey};

#[rustfmt::skip]
const SIGNATURE: &[u8] = &[
    0x48, 0x81, 0xEC, 0x40, 0x03, 0x00, 0x00, 0x84, 0xD2, 0x74, 0x05, 0xE8
];

#[rustfmt::skip]
const PATCH: &[u8] = &[
    0x48, 0x81, 0xEC, 0x40, 0x03, 0x00, 0x00, 0x84, 0xD2, 0x90, 0x90, 0xE8
];

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Input executable to patch
    input: Option<PathBuf>,

    /// Output file for patched executable
    output: Option<PathBuf>,

    /// Create a backup of the original executable
    #[arg(short, long)]
    backup: bool,
}

fn patch(input: &PathBuf, output: &PathBuf, do_backup: bool) {
    let mut binary = fs::read(input).expect("Could not read input file.");

    let offset = binary
        .windows(SIGNATURE.len())
        .position(|window| window == SIGNATURE)
        .expect("Could not find patch signature in binary.");

    println!("[+] Found signature at offset: 0x{:X}", offset);

    if &binary[offset..offset + PATCH.len()] == PATCH {
        println!("[!] Binary is already patched.");
        return;
    }

    if do_backup {
        let backup_path = input.with_extension("bak");
        fs::write(&backup_path, &binary).expect("Failed to create backup.");
        println!("[+] Backup created at: {}", backup_path.display());
    }

    binary[offset..offset + PATCH.len()].copy_from_slice(PATCH);
    fs::write(output, binary).expect("Could not write patched output.");
    println!("[+] Patch applied and written to: {}", output.display());
}

fn locate_default_installation() -> PathBuf {
    let path: String = RegKey::predef(HKEY_CLASSES_ROOT)
        .open_subkey("roblox-studio").unwrap()
        .open_subkey("DefaultIcon").unwrap()
        .get_value("").unwrap();

    PathBuf::from(path)
}

fn main() {
    let Cli { input, output, backup } = Cli::parse();

    let input = input.unwrap_or_else(locate_default_installation);
    let output = output.unwrap_or_else(|| {
        input.with_file_name("RobloxStudioBeta_INTERNAL.exe")
    });

    println!("[*] Input:  {}", input.display());
    println!("[*] Output: {}", output.display());

    let now = Instant::now();
    patch(&input, &output, backup);
    println!("[*] Done in {:?}.", now.elapsed());
}
