mod parser;
mod runner;
mod tag;

use std::path::{Path, PathBuf};

use crate::qc::parser::RawSeq;

pub fn dry_run(input: &Path, is_id: bool, is_rename: bool) {
    let reads: Vec<RawSeq> = parser::parse_input(input, is_id, is_rename);
    println!();
    reads.iter().for_each(|r| {
        log::info!("\x1b[0;32mID\t\t: {}\x1b[0m", r.id);
        log::info!("Read 1\t\t: {}", r.read_1.to_string_lossy());
        log::info!("Read 2\t\t: {}", r.read_2.to_string_lossy());

        match r.adapter_i7.as_ref() {
            Some(i7) => {
                log::info!("Adapter i5\t: {}", r.adapter_i5.as_ref().unwrap());
                log::info!("Adapter i7\t: {}", i7);
            }
            None => {
                if r.auto_idx {
                    log::info!("Adapter\t\t: AUTO-DETECT");
                } else {
                    log::info!("Adapter\t\t: {}", r.adapter_i5.as_ref().unwrap());
                }
            }
        };

        log::info!("Target Dir\t: {}", r.dir.to_string_lossy());
        if is_rename {
            log::info!("Target fname\t: {}", r.outname.as_ref().unwrap());
        }

        println!();
    });
}

pub fn process_input(
    input: &Path,
    is_id: bool,
    is_rename: bool,
    params: &Option<String>,
    outdir: &Option<PathBuf>,
) {
    let reads: Vec<RawSeq> = parser::parse_input(input, is_id, is_rename);
    runner::clean_reads(&reads, params, outdir);
}
