use std::fs;
use std::io::{self, Result, Write};
use std::path::Path;

use chrono::NaiveTime;
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm};
use indicatif::{ProgressBar, ProgressStyle};
use sysinfo::{System, SystemExt};

pub fn check_dir_exist(path: &Path) {
    if path.is_dir() {
        let selection = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("Output dir already exists: {}", path.display()))
            .interact();
        match selection {
            Ok(yes) => {
                if yes {
                    fs::remove_dir_all(path).expect("Failed removing the directory!");
                    println!();
                } else {
                    std::process::abort();
                }
            }
            Err(err) => panic!("Failed parsing user input: {}", err),
        }
    }
}

fn parse_duration(duration: u64) -> String {
    let sec = (duration % 60) as u32;
    let min = ((duration / 60) % 60) as u32;
    let hours = ((duration / 60) / 60) as u32;
    let time = NaiveTime::from_hms_opt(hours, min, sec);
    match time {
        Some(t) => t.format("%H:%M:%S").to_string(),
        None => String::from("00:00:00"),
    }
}

pub fn print_formatted_duration(duration: u64) {
    let time = parse_duration(duration);
    println!("{}: {}", "Execution time (HH:MM:SS)".yellow(), time);
}

pub fn set_spinner() -> ProgressBar {
    let spin = ProgressBar::new_spinner();
    spin.enable_steady_tick(150);
    spin.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("🌑🌒🌓🌔🌕🌖🌗🌘")
            .template("{spinner} {msg}"),
    );
    spin
}

pub fn get_system_info() -> Result<()> {
    let sys_info = sysinfo::System::new_all();
    let io = io::stdout();
    let mut handle = io::BufWriter::new(io);

    let total_ram = sys_info.get_total_memory();
    let gb = 1048576;

    writeln!(handle, "{}", "System Information".yellow())?;

    writeln!(
        handle,
        "{:18}: {} {}",
        "Operating system",
        get_os_name(&sys_info),
        get_os_version(&sys_info)
    )?;

    writeln!(
        handle,
        "{:18}: {}",
        "Kernel version",
        get_kernel_version(&sys_info)
    )?;
    writeln!(
        handle,
        "{:18}: {:?}",
        "Physical cores",
        num_cpus::get_physical()
    )?;
    writeln!(handle, "{:18}: {:?}", "Available threads", num_cpus::get())?;
    writeln!(handle, "{:18}: {} Gb", "Total RAM", total_ram / gb)?;
    writeln!(handle)?;

    Ok(())
}

fn get_os_name(sysinfo: &System) -> String {
    match sysinfo.get_name() {
        Some(i) => i,
        None => String::from("UNKNOWN"),
    }
}

fn get_os_version(sysinfo: &System) -> String {
    match sysinfo.get_os_version() {
        Some(i) => i,
        None => String::from(""),
    }
}

fn get_kernel_version(sysinfo: &System) -> String {
    match sysinfo.get_kernel_version() {
        Some(i) => i,
        None => String::from("UNKNOWN"),
    }
}

pub struct PrettyHeader {
    text: String,
    sym: char,
    len: usize,
    text_len: usize,
    sym_len: usize,
}

impl PrettyHeader {
    pub fn new(text: &str) -> Self {
        Self {
            text: String::from(text),
            sym: '=',
            len: 80,
            text_len: text.len(),
            sym_len: 0,
        }
    }

    pub fn get(&mut self) -> String {
        self.get_len();
        if self.text_len > self.len {
            self.text.yellow().to_string()
        } else {
            self.get_with_symbol().yellow().to_string()
        }
    }

    fn get_with_symbol(&mut self) -> String {
        let mut sym = self.get_symbols();
        let header = format!("Processing {}", self.text);
        if self.text_len % 2 != 0 {
            sym.push(self.sym);
        }

        format!("{} {} {}", sym, header, sym)
    }

    fn get_len(&mut self) {
        self.text_len = self.text.len();

        if self.len > self.text_len {
            self.sym_len = (self.len - self.text_len) / 2;
        } else {
            self.sym_len = self.len;
        }
    }

    fn get_symbols(&self) -> String {
        self.sym.to_string().repeat(self.sym_len)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn time_parsing_test() {
        let duration = 65;
        let duration_2 = 3600;
        let time = parse_duration(duration);
        let hours = parse_duration(duration_2);

        assert_eq!("00:01:05", time);
        assert_eq!("01:00:00", hours);
    }
}
