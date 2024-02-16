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
    println!("Execution time (HH:MM:SS): {}", time);
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

pub fn print_header(text: &str) {
    let header = format!("Processing {}", text);
    let length = 78;
    let sym = '=';
    let mut header = PrettyHeader::new(&header, sym, length);
    header.print_header().unwrap();
}

struct PrettyHeader {
    text: String,
    sym: char,
    len: usize,
    text_len: usize,
    sym_len: usize,
}

impl PrettyHeader {
    fn new(text: &str, sym: char, len: usize) -> Self {
        Self {
            text: String::from(text),
            sym,
            len,
            text_len: 0,
            sym_len: 0,
        }
    }

    fn print_header(&mut self) -> Result<()> {
        self.get_len();
        let io = io::stdout();
        let mut handle = io::BufWriter::new(io);
        if self.text_len > self.len {
            writeln!(handle, "{}", self.text.yellow())?;
        } else {
            self.print_with_symbol(&mut handle)?;
        }
        Ok(())
    }

    fn print_with_symbol<W: Write>(&mut self, handle: &mut W) -> Result<()> {
        self.print_symbols(handle);
        write!(handle, " {} ", self.text)?;
        self.print_symbols(handle);

        if self.text_len % 2 != 0 {
            write!(handle, "{}", self.sym)?;
        }

        writeln!(handle)?;
        Ok(())
    }

    fn get_len(&mut self) {
        self.text_len = self.text.len();

        if self.len > self.text_len {
            self.sym_len = (self.len - self.text_len) / 2;
        } else {
            self.sym_len = self.len;
        }
    }

    fn print_symbols<W: Write>(&self, io: &mut W) {
        (0..=self.sym_len).for_each(|_| {
            write!(io, "{}", self.sym).unwrap();
        });
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
