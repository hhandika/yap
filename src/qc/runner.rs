use std::fs;
use std::str;
use std::io::{self, Result, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[cfg(target_family="unix")]
use std::os::unix;

use spinners::{Spinner, Spinners};

use crate::parser::RawSeq;
use crate::utils;

pub fn check_fastp() {
    let out = Command::new("fastp")
        .arg("--version")
        .output();
        
        match out {
            Ok(out) =>  println!("[OK]\t{}\n", str::from_utf8(&out.stderr).unwrap().trim()),
            Err(_) => println!("[NOT FOUND]\tfastp"),
        }

}

pub fn clean_reads(reads: &[RawSeq], params: &Option<String>) {
    let dir = Path::new("clean_reads");
    check_dir_exists(&dir);
    reads.iter()
        .for_each(|read| {
            let mut run = Runner::new(&dir, read, params);

            if read.adapter_i7.as_ref().is_some() { // Check if i7 contains sequence
                run.dual_idx = true;
            }

            run.process_reads();
        });

    println!();
} 

fn check_dir_exists(dir: &Path) {
    if dir.exists() {
        panic!("{:?} DIR EXISTS. PLEASE RENAME OR REMOVE IT", dir);
    } else { // if not create one
        fs::create_dir_all(dir)
            .expect("CAN'T CREATE CLEAN READ DIR");
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
    fn new(
        dir: &Path, 
        input: &'a RawSeq, 
        params: &'a Option<String>
    ) -> Self {
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
        self.get_out_fnames(); 
        self.display_settings().unwrap();
        let spin = self.set_spinner();
        let out = self.call_fastp();
        
        let mut reports = FastpReports::new(&self.clean_dir);
        
        reports.check_fastp_status(&out);
        reports.write_stdout(&out);
        self.try_creating_symlink();
        reports.reorganize_reports().unwrap();
        spin.stop();
        self.print_done();
        reports.display_report_paths().unwrap();
    }

    fn print_done(&self) {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        writeln!(handle, "\x1b[0;32mDONE!\x1b[0m").unwrap();
    }

    fn get_out_fnames(&mut self) {
        let outdir = self.clean_dir.join("trimmed_reads");
        fs::create_dir_all(&outdir).unwrap();
        
        let out1 = self.reads.read_1.file_name().unwrap();
        let out2 = self.reads.read_2.file_name().unwrap();

        if self.is_rename() {
            let out1 = self.rename_output(&out1.to_str().unwrap());
            let out2 = self.rename_output(&out2.to_str().unwrap());
            self.out_r1 = outdir.join(out1);
            self.out_r2 = outdir.join(out2);
        } else {
            self.out_r1 = outdir.join(out1);
            self.out_r2 = outdir.join(out2);
        }
    }

    fn is_rename(&self) -> bool {
        self.reads.outname.is_some()
    }

    fn rename_output(&self, outname: &str) -> String {
        let target = self.reads.outname.as_ref().unwrap();
        outname.replace(&self.reads.id, &target)
    }

    fn display_settings(&self) -> Result<()> {
        let stdout = io::stdout();
        let mut buff = io::BufWriter::new(stdout);

        writeln!(buff, "Target dir\t: {}", &self.clean_dir.to_string_lossy())?;
        writeln!(buff, "Input R1\t: {}", &self.reads.read_1.to_string_lossy())?;
        writeln!(buff, "Input R2\t: {}", &self.reads.read_2.to_string_lossy())?;
        writeln!(buff, "Output R1\t: {}", &self.out_r1.to_string_lossy())?;
        writeln!(buff, "Output R2\t: {}", &self.out_r2.to_string_lossy())?;
        
        if self.reads.auto_idx {
            writeln!(buff, "Adapters\t: AUTO-DETECT")?;
        } else if !self.dual_idx {
            writeln!(buff, "Adapters\t: {}", self.reads.adapter_i5.as_ref().unwrap())?;
        } else {
            writeln!(buff, "Adapter i5\t: {}", self.reads.adapter_i5.as_ref().unwrap())?;
            writeln!(buff, "Adapters i7\t: {}", self.reads.adapter_i7.as_ref().unwrap())?;
        }

        writeln!(buff)?;

        Ok(())
    }

    fn set_spinner(&mut self) -> Spinner {
        let msg = "Fastp is processing...\t".to_string();
        
        Spinner::new(Spinners::Moon, msg)
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

        self.set_fastp_idx(&mut out);

        if self.params.is_some() {
            self.set_opt_params(&mut out);
        }
        
        out.output().unwrap()
    }

    fn set_fastp_idx(&self, out: &mut Command) {
        if self.dual_idx {
            self.set_fastp_dual_idx(out);
        } else if self.reads.auto_idx {
            self.set_fastp_auto_idx(out);
        } else {
            self.set_fastp_single_idx(out);
        }
    }

    fn set_fastp_auto_idx(&self, out: &mut Command) {
        out.arg("--detect_adapter_for_pe");
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
        out.arg(self.params.as_ref().unwrap());
    }

    fn try_creating_symlink(&self) {
        if cfg!(target_family="unix") {
            #[cfg(target_family="unix")]
            self.create_symlink().unwrap();
        } else {
            println!("Skip creating symlink in dir {} for {} and {}. \
                Operating system is not supported.", 
                &self.clean_dir.to_string_lossy(), 
                &self.reads.read_1.to_string_lossy(), 
                &self.reads.read_2.to_string_lossy());
        }
    }
    
    #[cfg(target_family="unix")]
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

    fn reorganize_reports(&mut self) -> Result<()>{
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

    fn display_report_paths(&self) -> Result<()>{
        let stdout = io::stdout();
        let mut handle = io::BufWriter::new(stdout);

        writeln!(handle)?;
        writeln!(handle, "Fastp Reports:")?;
        writeln!(handle, "1. {}", self.html_out.to_string_lossy())?;
        writeln!(handle, "2. {}", self.json_out.to_string_lossy())?;
        writeln!(handle, "3. {}", self.log_out.to_string_lossy())?;
        writeln!(handle)?;

        Ok(())
    }   
}