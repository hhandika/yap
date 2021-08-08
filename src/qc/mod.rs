mod parser;
mod runner;
mod tag;

use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::qc::parser::RawSeq;

pub fn dry_run(input: &Path, is_id: bool, is_rename: bool) {
    let reads: Vec<RawSeq> = parser::parse_input(input, is_id, is_rename);
    let stdout = io::stdout();
    let mut handle = io::BufWriter::new(stdout);

    writeln!(handle).unwrap();
    reads.iter().for_each(|r| {
        writeln!(handle, "\x1b[0;32mID\t\t: {}\x1b[0m", r.id).unwrap();
        writeln!(handle, "Read 1\t\t: {}", r.read_1.to_string_lossy()).unwrap();
        writeln!(handle, "Read 2\t\t: {}", r.read_2.to_string_lossy()).unwrap();

        match r.adapter_i7.as_ref() {
            Some(i7) => {
                writeln!(handle, "Adapter i5\t: {}", r.adapter_i5.as_ref().unwrap()).unwrap();
                writeln!(handle, "Adapter i7\t: {}", i7).unwrap();
            }
            None => {
                if r.auto_idx {
                    writeln!(handle, "Adapter\t\t: AUTO-DETECT").unwrap();
                } else {
                    writeln!(handle, "Adapter\t\t: {}", r.adapter_i5.as_ref().unwrap()).unwrap();
                }
            }
        };

        writeln!(handle, "Target Dir\t: {}", r.dir.to_string_lossy()).unwrap();
        if is_rename {
            writeln!(handle, "Target fname\t: {}", r.outname.as_ref().unwrap()).unwrap();
        }

        writeln!(handle).unwrap();
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
