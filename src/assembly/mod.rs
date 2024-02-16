pub mod cleaner;
mod finder;
mod parser;
mod runner;

use std::io::Result;
use std::path::PathBuf;

use colored::Colorize;

use crate::assembly::finder::SeqReads;
use crate::utils;

pub fn auto_process_input(
    path: &str,
    dirname: &str,
    threads: &Option<usize>,
    output_dir: &Option<PathBuf>,
    args: &Option<String>,
) {
    let samples = finder::auto_find_cleaned_fastq(path, dirname);
    runner::assemble_reads(&samples, threads, output_dir, args);
}

pub fn process_input(
    input: &str,
    threads: &Option<usize>,
    output_dir: &Option<PathBuf>,
    args: &Option<String>,
) {
    let dirs = parser::parse_sequence_dir(input);
    let samples = finder::find_cleaned_fastq(&dirs);
    runner::assemble_reads(&samples, threads, output_dir, args);
}

pub fn auto_dry_run(path: &str, dirname: &str) {
    let samples = finder::auto_find_cleaned_fastq(path, dirname);
    utils::get_system_info().unwrap();
    print_dry_run(&samples).unwrap();
}

pub fn dry_run(input: &str) {
    let dirs = parser::parse_sequence_dir(input);
    let samples = finder::find_cleaned_fastq(&dirs);
    utils::get_system_info().unwrap();
    print_dry_run(&samples).unwrap();
}

fn print_dry_run(dirs: &[SeqReads]) -> Result<()> {
    log::info!("{} {}", dirs.len(), "Total samples:".yellow());
    dirs.iter().for_each(|e| {
        log::info!("{:18}: {}", "ID".yellow(), e.id.yellow());
        log::info!("{:18}: {}", "Dir", e.dir.to_string_lossy());
        log::info!("{:18}: {}", "Read 1", e.read_1.to_string_lossy());
        log::info!("{:18}: {}", "Read 2", e.read_2.to_string_lossy());

        if e.singleton.is_some() {
            log::info!(
                "{:18}: {}",
                "Singleton",
                e.singleton.as_ref().unwrap().to_string_lossy()
            );
        }

        println!();
    });
    Ok(())
}
