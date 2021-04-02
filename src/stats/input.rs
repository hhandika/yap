//! Heru Handika
//! Module to process user inputs.

use std::path::PathBuf;
use std::sync::mpsc::channel;

use rayon::prelude::*;
use walkdir::WalkDir;

use crate::stats::fasta;
use crate::stats::fastq;
use crate::stats::output;
use crate::stats::sequence::{FastaStats, FastqStats};

pub fn process_wildcard(entries: &[&str], iscsv: bool, fastq: bool) {
    let files: Vec<PathBuf> = entries.iter().map(PathBuf::from).collect();
    par_process_files(&files, iscsv, fastq)
}

pub fn process_walkdir(path: &str, iscsv: bool, fastq: bool) {
    let entries = tranverse_dir(path, fastq);
    par_process_files(&entries, iscsv, fastq)
}

fn par_process_files(entries: &[PathBuf], iscsv: bool, fastq: bool) {
    if fastq {
        par_process_fastq(&entries, iscsv);
    } else {
        par_process_fasta(&entries, iscsv);
    }
}

fn tranverse_dir(path: &str, fastq: bool) -> Vec<PathBuf> {
    let mut entries = Vec::new();

    WalkDir::new(path)
        .into_iter()
        .filter_map(|ok| ok.ok())
        .filter(|e| e.file_type().is_file())
        .for_each(|e| {
            if fastq {
                let files = String::from(e.path().to_string_lossy());
                match_fastq(&files, &mut entries);
            } else {
                // then it is fasta
                let files = String::from(e.path().to_string_lossy());
                match_fasta(&files, &mut entries);
            }
        });

    entries
}

fn match_fastq(files: &str, entries: &mut Vec<PathBuf>) {
    match files {
        s if s.ends_with(".fastq.gz") => entries.push(PathBuf::from(files)),
        s if s.ends_with(".fq.gz") => entries.push(PathBuf::from(files)),
        s if s.ends_with("fastq.gzip") => entries.push(PathBuf::from(files)),
        s if s.ends_with("fq.gzip") => entries.push(PathBuf::from(files)),
        s if s.ends_with("fastq") => entries.push(PathBuf::from(files)),
        s if s.ends_with("fq") => entries.push(PathBuf::from(files)),
        _ => (),
    };
}

fn match_fasta(files: &str, entries: &mut Vec<PathBuf>) {
    match files {
        s if s.ends_with(".fasta.gz") => entries.push(PathBuf::from(files)),
        s if s.ends_with(".fas.gz") => entries.push(PathBuf::from(files)),
        s if s.ends_with(".fa.gz") => entries.push(PathBuf::from(files)),
        s if s.ends_with(".fasta.gzip") => entries.push(PathBuf::from(files)),
        s if s.ends_with(".fa.gzip") => entries.push(PathBuf::from(files)),
        s if s.ends_with(".fasta") => entries.push(PathBuf::from(files)),
        s if s.ends_with(".fas") => entries.push(PathBuf::from(files)),
        s if s.ends_with(".fa") => entries.push(PathBuf::from(files)),
        _ => (),
    };
}

fn par_process_fastq(files: &[PathBuf], iscsv: bool) {
    let (sender, receiver) = channel();

    files.into_par_iter().for_each_with(sender, |s, recs| {
        s.send(fastq::process_fastq(&recs)).unwrap();
    });
    let mut all_reads: Vec<FastqStats> = receiver.iter().collect();
    output::write_fastq(&mut all_reads, iscsv);
}

fn par_process_fasta(files: &[PathBuf], iscsv: bool) {
    let (sender, receiver) = channel();

    files.into_par_iter().for_each_with(sender, |s, recs| {
        s.send(fasta::process_fasta(&recs)).unwrap();
    });
    let mut all_reads: Vec<FastaStats> = receiver.iter().collect();
    output::write_fasta(&mut all_reads, iscsv);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tranverse_dir_test() {
        let input = "test_files/stats";
        let files = tranverse_dir(&input, true);

        assert_eq!(4, files.len())
    }

    #[test]
    fn match_fasta_test() {
        let input = vec!["test.fasta", "test.fas", "test.fa", "test.fa.gz"];
        let mut entries = Vec::new();

        input.iter().for_each(|e| {
            match_fasta(&e, &mut entries);
        });

        assert_eq!(4, entries.len());
    }

    #[test]
    fn match_fastq_test() {
        let input = vec!["test.fq", "test.fastq", "test.fq.gz", "test.fa.gz"];
        let mut entries = Vec::new();

        input.iter().for_each(|e| {
            match_fastq(&e, &mut entries);
        });

        assert_eq!(3, entries.len());
    }
}
