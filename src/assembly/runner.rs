use std::fs;
use std::io::{self, Result, Write};
#[cfg(not(target_os = "windows"))]
use std::os::unix;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use colored::Colorize;

use crate::assembly::finder::SeqReads;
use crate::utils;

pub fn assemble_reads(
    reads: &[SeqReads],
    threads: &Option<usize>,
    output_dir: &Option<PathBuf>,
    args: &Option<String>,
) {
    let dir = get_output_dir(output_dir);
    utils::check_dir_exist(&dir);
    fs::create_dir_all(&dir).expect("CAN'T CREATE ASSEMBLY DIR");
    let contig_dir = dir.join("contig_symlinks");
    fs::create_dir_all(&contig_dir).unwrap();
    log::info!("{:18} {}\n", "Total samples:".yellow(), reads.len());
    let sample_count = reads.len();
    let mut processed = 0;
    reads.iter().for_each(|r| {
        let mut run = Runner::new(&dir, &contig_dir, r, threads, args);
        run.run_spades();
        processed += 1;
        let processed_info = format!("Processed {} of {} samples", processed, sample_count);
        log::info!("{}", processed_info.blue());
        log::info!("");
    });

    log::info!("");
}

fn get_output_dir(output_dir: &Option<PathBuf>) -> PathBuf {
    match output_dir {
        Some(dir) => dir.clone(),
        None => PathBuf::from("assemblies"),
    }
}

struct Runner<'a> {
    reads: &'a SeqReads,
    output: PathBuf,
    symlink_dir: &'a Path,
    threads: &'a Option<usize>,
    args: &'a Option<String>,
}

impl<'a> Runner<'a> {
    fn new(
        dir: &Path,
        contig_dir: &'a Path,
        input: &'a SeqReads,
        threads: &'a Option<usize>,
        args: &'a Option<String>,
    ) -> Self {
        Self {
            reads: input,
            output: dir.join(&input.id),
            symlink_dir: contig_dir,
            threads,
            args,
        }
    }

    fn run_spades(&mut self) {
        utils::print_header(&self.reads.id);
        self.print_settings().unwrap();
        let spin = utils::set_spinner();
        spin.set_message("SPAdes is processing...");
        let out = self.call_spades();
        self.check_spades_success(&out);
        spin.finish_with_message(format!("{} SPAdes has finished", "✔".green()));
        self.create_symlink();
    }

    fn check_spades_success(&self, out: &Output) {
        if !out.status.success() {
            println!();
            io::stdout().write_all(&out.stdout).unwrap();
            io::stdout().write_all(&out.stderr).unwrap();
        }
    }

    fn call_spades(&self) -> Output {
        let mut out = Command::new("spades.py");
        out.arg("--pe1-1")
            .arg(&self.reads.read_1)
            .arg("--pe1-2")
            .arg(&self.reads.read_2)
            .arg("-o")
            .arg(&self.output.clone());
        self.get_spades_args(&mut out);

        if self.reads.singleton.is_some() {
            self.get_singleton(&mut out);
        }

        if self.threads.is_some() {
            self.get_thread_num(&mut out);
        }

        out.output().unwrap()
    }

    fn get_spades_args(&self, out: &mut Command) {
        if self.args.is_some() {
            self.get_opt_args(out);
        } else {
            self.get_default_args(out);
        }
    }
    fn get_default_args(&self, out: &mut Command) {
        out.arg("--careful");
    }

    fn get_opt_args(&self, out: &mut Command) {
        let args: Vec<&str> = self
            .args
            .as_ref()
            .expect("Invalid args")
            .split_whitespace()
            .collect();
        if args.len() > 1 {
            args.iter().for_each(|&a| {
                out.arg(a);
            });
        } else {
            out.arg(self.args.as_ref().unwrap());
        }
    }

    fn get_singleton(&self, out: &mut Command) {
        out.arg("--pe1-s")
            .arg(self.reads.singleton.as_ref().unwrap());
    }

    fn get_thread_num(&self, out: &mut Command) {
        out.arg("--threads")
            .arg(self.threads.as_ref().unwrap().to_string());
    }

    fn print_settings(&self) -> Result<()> {
        log::info!("{:18}: {}", "ID", &self.reads.id);
        log::info!("{:18}: {}", "Input dir", &self.reads.dir.to_string_lossy());
        log::info!(
            "{:18}: {}",
            "Input R2",
            &self
                .reads
                .read_2
                .file_name()
                .expect("NO FILE")
                .to_string_lossy()
        );
        log::info!(
            "{:18}: {}",
            "Input R1",
            &self
                .reads
                .read_1
                .file_name()
                .expect("NO FILE")
                .to_string_lossy()
        );

        if self.reads.singleton.is_some() {
            log::info!(
                "{:18}: {}",
                "Singleton",
                &self
                    .reads
                    .singleton
                    .as_ref()
                    .expect("NO FILE")
                    .to_string_lossy()
            );
        }

        log::info!("{:18}: {}", "Output", &self.output.display());

        if self.args.is_some() {
            log::info!(
                "{:18}: {}",
                "Opt params",
                &self.args.as_ref().expect("NO ARGS")
            );
        }
        log::info!("");

        Ok(())
    }

    fn create_symlink(&self) {
        let contig_sym = format!("{}_contigs.fasta", self.reads.id);
        let contigs_path = self.output.join("contigs.fasta");

        if contigs_path.is_file() {
            #[cfg(not(target_os = "windows"))]
            let path = contigs_path.canonicalize().expect("NO FILE");
            let symlink = self.symlink_dir.join(contig_sym);
            #[cfg(not(target_os = "windows"))]
            unix::fs::symlink(path, &symlink).expect("CAN'T CREATE SYMLINK");
            self.print_contig_path(&contigs_path, &symlink);
        } else {
            log::warn!(
                "{} \
                SPAdes HAS FAILED. PLEASE CHECK SPAdes OUTPUT ABOVE FOR DETAILS.\n",
                "[ERROR]".red()
            );
        }
    }

    fn print_contig_path(&self, path: &Path, symlink: &Path) {
        println!();
        log::info!("{}", "Contig Path".yellow());
        log::info!("{:18}: {}", "File", path.display());
        log::info!("{:18}: {}", "Symlink", symlink.display());
        println!();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn output_dir_test() {
        let path = PathBuf::from("test/assemblies/");
        let output = Some(path.clone());
        let output_dir = get_output_dir(&output);

        assert_eq!(PathBuf::from(&path), output_dir);
    }
}
