use std::fs;
use std::io::{self, Result, Write};
#[cfg(not(target_os = "windows"))]
use std::os::unix::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use crate::assembly::finder::SeqReads;
use crate::utils;

pub fn assemble_reads(
    reads: &[SeqReads],
    threads: &Option<usize>,
    outdir: &Option<PathBuf>,
    args: &Option<String>,
) {
    let dir = get_outdir(&outdir);
    utils::check_dir_exist(&dir);
    fs::create_dir_all(&dir).expect("CAN'T CREATE ASSEMBLY DIR");
    let contig_dir = dir.join("contig_symlinks");
    fs::create_dir_all(&contig_dir).unwrap();
    println!("\x1b[0;33mTotal samples: {}\n\x1b[0m", reads.len());
    reads.iter().for_each(|r| {
        let mut run = Runner::new(&dir, &contig_dir, r, threads, args);
        run.run_spades();
    });
}

fn get_outdir(outdir: &Option<PathBuf>) -> PathBuf {
    match outdir {
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
        spin.set_message("SPAdes is processing...\t");
        let out = self.call_spades();
        self.check_spades_success(&out);
        spin.finish_with_message("DONE!");
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
        out.arg(self.args.as_ref().unwrap());
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
        log::info!("ID\t\t: {}", &self.reads.id);
        log::info!("Input R1\t: {}", &self.reads.read_1.to_string_lossy());
        log::info!("Input R2\t: {}", &self.reads.read_2.to_string_lossy());

        if self.reads.singleton.is_some() {
            log::info!(
                "Singleton\t: {}",
                &self.reads.singleton.as_ref().unwrap().to_string_lossy()
            );
        }

        log::info!("Output\t\t: {}", &self.output.to_string_lossy());

        if self.args.is_some() {
            log::info!("Opt params\t: {}", &self.args.as_ref().unwrap());
        }
        println!();

        Ok(())
    }

    fn create_symlink(&self) {
        let contig_sym = format!("{}_contigs.fasta", self.reads.id);
        let contigs_path = self.output.join("contigs.fasta");

        if contigs_path.is_file() {
            #[cfg(not(target_os = "windows"))]
            let path = contigs_path.canonicalize().unwrap();
            let symlink = self.symlink_dir.join(contig_sym);
            #[cfg(not(target_os = "windows"))]
            fs::symlink(&path, &symlink).unwrap();
            self.print_contig_path(&contigs_path, &symlink);
        } else {
            log::warn!(
                "\x1b[41m[ERROR]\x1b[0m \
                SPAdes HAS FAILED. PLEASE CHECK SPAdes OUTPUT ABOVE FOR DETAILS.\n"
            );
        }
    }

    fn print_contig_path(&self, path: &Path, symlink: &Path) {
        println!();
        log::info!("Contig Path");
        log::info!("File\t\t: {}", path.to_string_lossy());
        log::info!("Symlink\t\t: {}", symlink.to_string_lossy());
        println!();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn outdir_test() {
        let path = PathBuf::from("test/assemblies/");
        let output = Some(path.clone());
        let outdir = get_outdir(&output);

        assert_eq!(PathBuf::from(&path), outdir);
    }
}
