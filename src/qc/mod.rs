mod parser;
mod runner;
mod tag;

use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::utils;
use colored::Colorize;

use crate::qc::parser::RawSeq;
use crate::qc::runner::Fastp;

pub struct Qc<'a> {
    pub input: &'a Path,
    pub is_id: bool,
    pub is_rename: bool,
    pub params: Option<&'a str>,
    pub output_dir: Option<&'a Path>,
}

impl<'a> Qc<'a> {
    pub fn new(
        input: &'a Path,
        is_id: bool,
        is_rename: bool,
        params: Option<&'a str>,
        output_dir: Option<&'a Path>,
    ) -> Self {
        Self {
            input,
            is_id,
            is_rename,
            params,
            output_dir,
        }
    }

    pub fn dry_run(&self) {
        let reads: Vec<RawSeq> = parser::parse_input(self.input, self.is_id, self.is_rename);
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
            if self.is_rename {
                log::info!("{:18}: {}", "Target fname", r.output_name.as_ref().unwrap());
            }

            println!();
        });
    }

    pub fn run(&self) {
        let reads: Vec<RawSeq> = parser::parse_input(&self.input, self.is_id, self.is_rename);
        self.clean_reads(&reads);
    }

    pub fn clean_reads(&self, reads: &[RawSeq]) {
        let dir = self.get_output_dir();
        utils::check_dir_exist(&dir);
        fs::create_dir_all(&dir).expect("CAN'T CREATE CLEAN READ DIR");
        let sample_count = reads.len();
        let mut processed = 0;
        reads.iter().for_each(|read| {
            let mut runner = Fastp::new(&dir, read, self.params);

            if read.adapter_i7.as_ref().is_some() {
                // Check if i7 contains sequence
                runner.dual_idx = true;
            }

            runner.run();
            processed += 1;
            log::info!("Processed {} of {} samples", processed, sample_count);
            log::info!("");
        });

        log::info!("");
    }

    fn get_output_dir(&self) -> PathBuf {
        match self.output_dir {
            Some(dir) => dir.to_path_buf(),
            None => PathBuf::from("clean_reads"),
        }
    }
}
