use std::io::Result;
use std::process::Command;
use std::str;

use colored::Colorize;

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
        Ok(out) => log::info!(
            "{:18}: {}",
            str::from_utf8(&out.stderr).unwrap().trim(),
            "[OK]".green(),
        ),
        Err(_) => log::info!("{:18}: {}", "fastp", "[NOT FOUND]".red()),
    }
}

fn check_spades() {
    let out = Command::new("spades.py").arg("--version").output();
    match out {
        Ok(out) => log::info!(
            "{:18} {}",
            str::from_utf8(&out.stdout).unwrap().trim(),
            "[OK]".green(),
        ),
        Err(_) => log::info!("{:18}: {}", "SPAdes", "[NOT FOUND]".red()),
    }
}
