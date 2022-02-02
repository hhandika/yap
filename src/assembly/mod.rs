pub mod cleaner;
mod finder;
mod parser;
mod runner;

use std::io::{self, Result, Write};
use std::path::PathBuf;

use crate::assembly::finder::SeqReads;
use crate::utils;

pub fn auto_process_input(
    path: &str,
    dirname: &str,
    threads: &Option<usize>,
    outdir: &Option<PathBuf>,
    args: &Option<String>,
) {
    let samples = finder::auto_find_cleaned_fastq(path, dirname);
    runner::assemble_reads(&samples, threads, outdir, args);
}

pub fn process_input(
    input: &str,
    threads: &Option<usize>,
    outdir: &Option<PathBuf>,
    args: &Option<String>,
) {
    let dirs = parser::parse_seqdir(input);
    let samples = finder::find_cleaned_fastq(&dirs);
    runner::assemble_reads(&samples, threads, outdir, args);
}

pub fn auto_dryrun(path: &str, dirname: &str) {
    let samples = finder::auto_find_cleaned_fastq(path, dirname);
    utils::get_system_info().unwrap();
    print_dryrun(&samples).unwrap();
}

pub fn dryrun(input: &str) {
    let dirs = parser::parse_seqdir(input);
    let samples = finder::find_cleaned_fastq(&dirs);
    utils::get_system_info().unwrap();
    print_dryrun(&samples).unwrap();
}

fn print_dryrun(dirs: &[SeqReads]) -> Result<()> {
    log::info!("\x1b[0;33mTotal samples: {}\n\x1b[0m", dirs.len());
    dirs.iter().for_each(|e| {
        log::info!("\x1b[0;32mID\t\t: {}\x1b[0m", e.id);
        log::info!("Dir\t\t: {}", e.dir.to_string_lossy());
        log::info!("Read 1\t\t: {}", e.read_1.to_string_lossy());
        log::info!("Read 2\t\t: {}", e.read_2.to_string_lossy());

        if e.singleton.is_some() {
            log::info!(
                "Singleton\t: {}",
                e.singleton.as_ref().unwrap().to_string_lossy()
            );
        }

        println!();
    });
    Ok(())
}
