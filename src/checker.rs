use std::io::{self, Result, Write};
use std::process::Command;
use std::str;

use crate::utils;

pub fn check_dependencies() -> Result<()> {
    let stdout = io::stdout();
    let mut handle = io::BufWriter::new(stdout);
    utils::get_system_info().unwrap();
    writeln!(handle, "Dependencies:")?;
    check_fastp(&mut handle)?;
    check_spades(&mut handle)?;
    writeln!(handle)?;
    Ok(())
}

fn check_fastp<W: Write>(handle: &mut W) -> Result<()> {
    let out = Command::new("fastp").arg("--version").output();

    match out {
        Ok(out) => writeln!(
            handle,
            "[OK]\t{}",
            str::from_utf8(&out.stderr).unwrap().trim()
        )?,
        Err(_) => writeln!(handle, "[NOT FOUND]\tfastp")?,
    }

    Ok(())
}

fn check_spades<W: Write>(handle: &mut W) -> Result<()> {
    let out = Command::new("spades.py").arg("--version").output();
    match out {
        Ok(out) => writeln!(
            handle,
            "[OK]\t{}",
            str::from_utf8(&out.stdout).unwrap().trim()
        )?,
        Err(_) => writeln!(handle, "[NOT FOUND]\tSPAdes")?,
    }

    Ok(())
}
