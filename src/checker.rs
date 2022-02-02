use std::io::Result;
use std::process::Command;
use std::str;

use crate::utils;

pub fn check_dependencies() -> Result<()> {
    utils::get_system_info().unwrap();
    log::info!("Dependencies:");
    check_fastp();
    check_spades();
    println!();
    Ok(())
}

fn check_fastp() {
    let out = Command::new("fastp").arg("--version").output();

    match out {
        Ok(out) => log::info!("[OK]\t{}", str::from_utf8(&out.stderr).unwrap().trim()),
        Err(_) => log::info!("\x1b[0;41m[NOT FOUND]\x1b[0m\tfastp"),
    }
}

fn check_spades() {
    let out = Command::new("spades.py").arg("--version").output();
    match out {
        Ok(out) => log::info!("[OK]\t{}", str::from_utf8(&out.stdout).unwrap().trim()),
        Err(_) => log::info!("\x1b[0;41m[NOT FOUND]\x1b[0m\tSPAdes"),
    }
}
