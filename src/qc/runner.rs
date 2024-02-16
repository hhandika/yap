use std::fs;
use std::io::{self, BufWriter, Result, Write};
// use std::os::unix::process;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[cfg(target_family = "unix")]
use std::os::unix;

use crate::qc::parser::RawSeq;
use crate::utils;

pub fn clean_reads(reads: &[RawSeq], params: &Option<String>, output_dir: &Option<PathBuf>) {
    let dir = get_output_dir(output_dir);
    utils::check_dir_exist(&dir);
    fs::create_dir_all(&dir).expect("CAN'T CREATE CLEAN READ DIR");
    let sample_count = reads.len();
    let mut processed = 0;
    reads.iter().for_each(|read| {
        let mut run = Runner::new(&dir, read, params);

        if read.adapter_i7.as_ref().is_some() {
            // Check if i7 contains sequence
            run.dual_idx = true;
        }

        run.process_reads();
        processed += 1;
    });

    log::info!("");
    log::info!("Processed {} of {} samples", processed, sample_count);
    log::info!("");
}

fn get_output_dir(output_dir: &Option<PathBuf>) -> PathBuf {
    match output_dir {
        Some(dir) => dir.clone(),
        None => PathBuf::from("clean_reads"),
    }
}

struct Runner<'a> {
    clean_dir: PathBuf,
    dual_idx: bool,
    out_r1: PathBuf,
    out_r2: PathBuf,
    reads: &'a RawSeq,
    params: &'a Option<String>,
}

impl<'a> Runner<'a> {
    fn new(dir: &Path, input: &'a RawSeq, params: &'a Option<String>) -> Self {
        Self {
            clean_dir: dir.join(&input.dir),
            dual_idx: false,
            out_r1: PathBuf::new(),
            out_r2: PathBuf::new(),
            reads: input,
            params,
        }
    }

    fn process_reads(&mut self) {
        utils::print_header(&self.reads.id);
        self.get_output_filename();
        self.display_settings();
        let spin = utils::set_spinner();
        spin.set_message("Fastp is processing\t");
        let out = self.call_fastp();
        let mut reports = FastpReports::new(&self.clean_dir);
        reports.check_fastp_status(&out);
        reports.write_stdout(&out);
        self.try_creating_symlink();
        reports.reorganize_reports().unwrap();
        spin.finish_with_message("\x1b[0;32mDONE!\x1b[0m");
        reports.display_report_paths();
    }

    fn get_output_filename(&mut self) {
        let output_dir = self.clean_dir.join("trimmed_reads");
        fs::create_dir_all(&output_dir).unwrap();
        let out1 = self.reads.read_1.file_name().unwrap();
        let out2 = self.reads.read_2.file_name().unwrap();

        if self.is_rename() {
            let out1 = self.rename_output(&out1.to_str().unwrap());
            let out2 = self.rename_output(&out2.to_str().unwrap());
            self.out_r1 = output_dir.join(out1);
            self.out_r2 = output_dir.join(out2);
        } else {
            self.out_r1 = output_dir.join(out1);
            self.out_r2 = output_dir.join(out2);
        }
    }

    fn is_rename(&self) -> bool {
        self.reads.output_name.is_some()
    }

    fn rename_output(&self, output_name: &str) -> String {
        let target = self.reads.output_name.as_ref().unwrap();
        output_name.replace(&self.reads.id, &target)
    }

    fn display_settings(&self) {
        log::info!("Target dir\t: {}", &self.clean_dir.to_string_lossy());
        log::info!(
            "Input dir\t: {}",
            &self.reads.read_1.parent().unwrap().to_string_lossy()
        );
        log::info!(
            "Input R1\t: {}",
            &self.reads.read_1.file_name().unwrap().to_string_lossy()
        );
        log::info!(
            "Input R2\t: {}",
            &self.reads.read_2.file_name().unwrap().to_string_lossy()
        );
        log::info!(
            "Output Dir\t: {}",
            &self.out_r1.parent().unwrap().to_string_lossy()
        );
        log::info!(
            "Output R1\t: {}",
            &self.out_r1.file_name().unwrap().to_string_lossy()
        );
        log::info!(
            "Output R2\t: {}",
            &self.out_r2.file_name().unwrap().to_string_lossy()
        );
        if self.reads.auto_idx {
            log::info!("Adapters\t: AUTO-DETECT");
        } else if !self.dual_idx {
            log::info!("Adapters\t: {}", self.reads.adapter_i5.as_ref().unwrap());
        } else {
            log::info!("Adapter i5\t: {}", self.reads.adapter_i5.as_ref().unwrap());
            log::info!("Adapters i7\t: {}", self.reads.adapter_i7.as_ref().unwrap());
        }

        log::info!("");
    }

    fn call_fastp(&self) -> Output {
        let mut out = Command::new("fastp");

        out.arg("-i")
            .arg(self.reads.read_1.clone())
            .arg("-I")
            .arg(self.reads.read_2.clone())
            .arg("-o")
            .arg(self.out_r1.clone())
            .arg("-O")
            .arg(self.out_r2.clone());
        if !self.reads.auto_idx {
            self.set_fastp_idx(&mut out)
        }
        self.set_opt_params(&mut out);

        out.output().unwrap()
    }

    fn set_fastp_idx(&self, out: &mut Command) {
        if self.dual_idx {
            self.set_fastp_dual_idx(out);
        } else {
            self.set_fastp_single_idx(out);
        }
    }

    fn set_fastp_single_idx(&self, out: &mut Command) {
        out.arg("--adapter_sequence")
            .arg(String::from(self.reads.adapter_i5.as_ref().unwrap()));
    }

    fn set_fastp_dual_idx(&self, out: &mut Command) {
        out.arg("--adapter_sequence")
            .arg(String::from(self.reads.adapter_i5.as_ref().unwrap()))
            .arg("--adapter_sequence_r2")
            .arg(String::from(self.reads.adapter_i7.as_ref().unwrap()));
    }

    fn set_opt_params(&self, out: &mut Command) {
        match self.params {
            Some(param) => {
                let params: Vec<&str> = param.split_whitespace().collect();
                params.iter().for_each(|param| {
                    out.arg(param);
                });
            }
            None => (),
        }
    }

    fn try_creating_symlink(&self) {
        if cfg!(target_family = "unix") {
            #[cfg(target_family = "unix")]
            self.create_symlink().unwrap();
        } else {
            println!(
                "Skip creating symlink in dir {} for {} and {}. \
                Operating system is not supported.",
                &self.clean_dir.to_string_lossy(),
                &self.reads.read_1.to_string_lossy(),
                &self.reads.read_2.to_string_lossy()
            );
        }
    }

    #[cfg(target_family = "unix")]
    fn create_symlink(&self) -> Result<()> {
        let symdir = self.clean_dir.join("raw_read_symlinks");
        fs::create_dir_all(&symdir)?;
        let abs_r1 = self.reads.read_1.canonicalize().unwrap();
        let abs_r2 = self.reads.read_2.canonicalize().unwrap();
        let path_r1 = symdir.join(self.reads.read_1.file_name().unwrap());
        let path_r2 = symdir.join(self.reads.read_2.file_name().unwrap());
        unix::fs::symlink(abs_r1, path_r1)?;
        unix::fs::symlink(abs_r2, path_r2)?;
        Ok(())
    }
}

struct FastpReports {
    dir: PathBuf,
    html: PathBuf,
    json: PathBuf,
    log: PathBuf,
    html_out: PathBuf,
    json_out: PathBuf,
    log_out: PathBuf,
}

impl FastpReports {
    fn new(dir: &Path) -> Self {
        Self {
            dir: dir.join("fastp_reports"),
            html: PathBuf::from("fastp.html"),
            json: PathBuf::from("fastp.json"),
            log: PathBuf::from("fastp.log"),
            html_out: PathBuf::new(),
            json_out: PathBuf::new(),
            log_out: PathBuf::new(),
        }
    }

    // Less likely this will be called
    // because potential input errors that cause fastp
    // to failed is mitigated before passing the input
    // to it.
    fn check_fastp_status(&self, out: &Output) {
        if !out.status.success() {
            self.fastp_is_failed(out);
        }

        if !self.html.is_file() || !self.json.is_file() {
            self.fastp_is_failed(out);
        }
    }
    fn fastp_is_failed(&self, out: &Output) {
        io::stdout().write_all(&out.stdout).unwrap();
        io::stdout().write_all(&out.stderr).unwrap();
        panic!("FASTP FAILED TO RUN");
    }

    // We remove the clutter of fastp stdout in the console.
    // Instead, we save it as a log file.
    fn write_stdout(&self, out: &Output) {
        let fname = fs::File::create(&self.log).unwrap();
        let mut buff = BufWriter::new(&fname);

        // Rust recognize fastp console output as stderr
        // Hence, we write stderr instead of stdout.
        buff.write_all(&out.stderr).unwrap();
    }

    fn reorganize_reports(&mut self) -> Result<()> {
        fs::create_dir_all(&self.dir)?;
        self.html_out = self.dir.join(&self.html);
        self.json_out = self.dir.join(&self.json);
        self.log_out = self.dir.join(&self.log);
        // Move json, html, and log reports
        fs::rename(&self.html, &self.html_out)?;
        fs::rename(&self.json, &self.json_out)?;
        fs::rename(&self.log, &self.log_out)?;

        Ok(())
    }

    fn display_report_paths(&self) {
        log::info!("");
        log::info!("Fastp Reports:");
        log::info!("1. {}", self.html_out.to_string_lossy());
        log::info!("2. {}", self.json_out.to_string_lossy());
        log::info!("3. {}", self.log_out.to_string_lossy());
        log::info!("");
    }
}
