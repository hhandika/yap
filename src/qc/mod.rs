mod parser;
mod runner;
mod tag;

use std::path::{Path, PathBuf};

use colored::Colorize;

use crate::qc::parser::RawSeq;

pub fn dry_run(input: &Path, is_id: bool, is_rename: bool) {
    let reads: Vec<RawSeq> = parser::parse_input(input, is_id, is_rename);
    println!();
    reads.iter().for_each(|r| {
        log::info!("{:18}: {}\x1b[0m", "ID".yellow(), r.id.yellow());
        log::info!("{:18}: {}", "Read 1", r.read_1.to_string_lossy());
        log::info!("{:18}: {}", "Read 2", r.read_2.to_string_lossy());

        match r.adapter_i7.as_ref() {
            Some(i7) => {
                log::info!("{:18}: {}", "Adapter i5", r.adapter_i5.as_ref().unwrap());
                log::info!("{:18}: {}", "Adapter i7", i7);
            }
            None => {
                if r.auto_idx {
                    log::info!("{:18}: AUTO-DETECT", "Adapter");
                } else {
                    log::info!("{:18}: {}", "Adapter", r.adapter_i5.as_ref().unwrap());
                }
            }
        };

        log::info!("{:18}: {}", "Target Dir", r.dir.to_string_lossy());
        if is_rename {
            log::info!("{:18}: {}", "Target fname", r.output_name.as_ref().unwrap());
        }

        println!();
    });
}

pub fn process_input(
    input: &Path,
    is_id: bool,
    is_rename: bool,
    params: &Option<String>,
    output_dir: &Option<PathBuf>,
) {
    let reads: Vec<RawSeq> = parser::parse_input(input, is_id, is_rename);
    runner::clean_reads(&reads, params, output_dir);
}
