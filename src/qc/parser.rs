use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::PathBuf;

use glob::{glob_with, MatchOptions};

use crate::qc::tag;

pub struct RawSeq {
    pub id: String,
    pub dir: PathBuf,
    pub read_1: PathBuf,
    pub read_2: PathBuf,
    pub adapter_i5: Option<String>,
    pub adapter_i7: Option<String>,
    pub outname: Option<String>,
    pub auto_idx: bool,
    pub command: Option<String>,
}

impl RawSeq {
    pub fn new() -> Self {
        Self {
            id: String::new(),
            dir: PathBuf::new(),
            read_1: PathBuf::new(),
            read_2: PathBuf::new(),
            adapter_i5: None,
            adapter_i7: None,
            outname: None,
            auto_idx: false,
            command: None,
        }
    }

    fn get_id(&mut self, id: &str) {
        self.id = String::from(id);
    }

    fn get_dir(&mut self, is_id: bool, is_rename: bool) {
        if !is_id && !is_rename {
            self.dir = PathBuf::from(&self.id);
        } else if is_rename {
            self.dir = PathBuf::from(&self.outname.as_ref().unwrap());
        } else {
            self.create_dir_from_r1();
        }
    }

    fn create_dir_from_r1(&mut self) {
        let fnames = String::from(
            self.read_1
                .file_name()
                .expect("MISSING FILES")
                .to_string_lossy(),
        );

        let ids = split_strings(&fnames, false);
        let dir = format!("{}_{}_{}", ids[0], ids[1], ids[2]);
        self.dir = PathBuf::from(dir);
    }

    fn get_reads(&mut self, reads: &[PathBuf]) {
        reads
            .iter()
            .for_each(|reads| match reads.to_string_lossy().to_uppercase() {
                s if s.contains("READ1") => self.read_1 = PathBuf::from(reads),
                s if s.contains("_R1") => self.read_1 = PathBuf::from(reads),
                s if s.contains("READ2") => self.read_2 = PathBuf::from(reads),
                s if s.contains("_R2") => self.read_2 = PathBuf::from(reads),
                _ => (),
            });

        self.check_missing_reads();
    }

    fn check_missing_reads(&self) {
        let missing_r1 = self.read_1.to_string_lossy().is_empty();
        let missing_r2 = self.read_2.to_string_lossy().is_empty();
        if missing_r1 || missing_r2 {
            panic!(
                "CANNOT FIND BOTH READS FOR {}. \
                Read 1: {:?} \
                Read 2: {:?}",
                self.id, self.read_1, self.read_2
            );
        }
    }

    fn get_adapter_single(&mut self, adapter: &str) {
        self.adapter_i5 = Some(String::from(adapter));
    }

    fn get_adapter_dual(&mut self, adapter_i5: &str, adapter_i7: &str) {
        let i5 = String::from(adapter_i5.trim());
        let i7 = String::from(adapter_i7.trim());

        if self.is_both_idx_exist(&i5, &i7) {
            self.adapter_i5 = Some(i5);
            self.adapter_i7 = Some(i7);
        } else if self.is_missing_i7(&i5, &i7) {
            self.adapter_i5 = Some(i5);
        } else if self.is_missing_both_idx(&i5, &i7) {
            self.get_adapter_auto();
        } else {
            self.adapter_i5 = Some(i5);
        }
    }

    fn is_both_idx_exist(&self, i5: &str, i7: &str) -> bool {
        !i5.is_empty() && !i7.is_empty()
    }

    fn is_missing_i7(&self, i5: &str, i7: &str) -> bool {
        !i5.is_empty() && i7.is_empty()
    }

    fn is_missing_both_idx(&self, i5: &str, i7: &str) -> bool {
        i5.is_empty() && i7.is_empty()
    }

    fn get_adapter_auto(&mut self) {
        self.auto_idx = true;
    }

    fn get_output_name(&mut self, fname: &str) {
        self.outname = Some(fname.to_string());
    }

    // fn get_costum_commands(&mut self, command: &str) {

    // }
}

pub fn parse_csv(input: &PathBuf, is_id: bool, is_rename: bool) -> Vec<RawSeq> {
    let file = File::open(input).unwrap();
    let buff = BufReader::new(file);

    let mut raw_seqs = Vec::new();
    let mut lcounts: usize = 0;

    buff.lines()
        .filter_map(|ok| ok.ok())
        .skip(1)
        .for_each(|line| {
            let mut seq = RawSeq::new();
            let lines = split_strings(&line, true);
            let id = String::from(&lines[0]);
            let reads = glob_raw_reads(&input, &id, is_id);
            check_reads(&reads, &id);
            seq.get_id(&id);
            seq.get_reads(&reads);
            if is_rename {
                get_adapter_rename(&mut seq, &lines);
            } else {
                get_adapters(&mut seq, &lines);
            }

            seq.get_dir(is_id, is_rename);
            raw_seqs.push(seq);
            lcounts += 1;
        });

    println!("Total samples: {}", lcounts);

    raw_seqs
}

fn check_reads(reads: &[PathBuf], id: &str) {
    match reads.len() {
        0 => panic!(
            "CANNOT FIND FILE {}. \
                USE THE --id FLAG IF YOU USE THE FILE ID.",
            id
        ),
        2 => (),
        _ => panic!("REQUIRED TWO READS FOR {}. FOUND: {:?}", id, reads),
    }
}

fn get_adapters(seq: &mut RawSeq, adapters: &[String]) {
    match adapters.len() {
        1 => seq.get_adapter_auto(),
        2 => get_adapter_single(seq, &adapters[1]),
        3 => get_adapter_dual(seq, &adapters[1], &adapters[2]),
        4 => get_insert_single(seq, &adapters[1], &adapters[2], &adapters[3]),
        5 => get_insert_dual(seq, &adapters[1], &adapters[2], &adapters[3], &adapters[4]),
        _ => panic!(
            "Unexpected cvs columns. It should be \
            2 columns for single index and 3 column for \
            dual index. The app received {} columns",
            adapters.len()
        ),
    }
}

fn get_adapter_rename(seq: &mut RawSeq, adapters: &[String]) {
    match adapters.len() {
        1 => panic!("MISSING AN OUTPUT NAME COLUMN"),
        2 => {
            seq.get_output_name(&adapters[1]);
            seq.get_adapter_auto();
        }

        3 => {
            seq.get_output_name(&adapters[1]);
            get_adapter_single(seq, &adapters[2]);
        }

        4 => {
            seq.get_output_name(&adapters[1]);
            get_adapter_dual(seq, &adapters[2], &adapters[3]);
        }
        5 => {
            seq.get_output_name(&adapters[1]);
            get_insert_single(seq, &adapters[2], &adapters[3], &adapters[4]);
        }
        6 => {
            seq.get_output_name(&adapters[1]);
            get_insert_dual(seq, &adapters[2], &adapters[3], &adapters[4], &adapters[5]);
        }

        _ => panic!("TOO MANY COLUMN. SIX MAX FOR RENAMING"),
    }
}

fn get_adapter_single(seq: &mut RawSeq, adapters: &str) {
    let i5 = adapters.to_uppercase();
    if is_insert_missing(&i5) {
        panic!("INSERT MISSING!");
    } else {
        seq.get_adapter_single(&i5);
    }
}

fn get_adapter_dual(seq: &mut RawSeq, i5: &str, i7: &str) {
    let adapter_i5 = i5.to_uppercase();
    if is_insert_missing(&adapter_i5) {
        // i7 is a tag
        let adapter_i5 = tag::insert_tag(i5, i7);
        seq.get_adapter_single(&adapter_i5);
    } else {
        let adapter_i7 = i7.to_uppercase();
        seq.get_adapter_dual(&adapter_i5, &adapter_i7);
    }
}

fn get_insert_single(seq: &mut RawSeq, i5: &str, i7: &str, insert: &str) {
    let adapter_i7 = i7.to_uppercase();
    if is_insert_missing(i5) {
        let adapter_i5 = tag::insert_tag(i5, insert);
        seq.get_adapter_dual(&adapter_i5, &adapter_i7);
    } else {
        panic!("INVALID COLUMNS FOR {}!", seq.id);
    }
}

fn get_insert_dual(seq: &mut RawSeq, i5: &str, i7: &str, in_i5: &str, in_i7: &str) {
    let i5 = tag::insert_tag(i5, in_i5);
    let i7 = tag::insert_tag(i7, in_i7);
    seq.get_adapter_dual(&i5, &i7);
}

fn is_insert_missing(adapter: &str) -> bool {
    adapter.contains('*')
}

fn split_strings(lines: &str, csv: bool) -> Vec<String> {
    let mut sep = ',';
    if !csv {
        sep = '_';
    }
    let seqs = lines.split(sep).map(|e| e.trim().to_string()).collect();
    seqs
}

fn glob_raw_reads(path: &PathBuf, id: &str, is_id: bool) -> Vec<PathBuf> {
    let patterns = get_patterns(path, id, is_id);
    let opts = MatchOptions {
        case_sensitive: true,
        ..Default::default()
    };

    glob_with(&patterns, opts)
        .unwrap()
        .filter_map(|ok| ok.ok())
        .collect()
}

fn get_patterns(path: &PathBuf, id: &str, is_id: bool) -> String {
    let parent = path.parent().unwrap();
    let mut pat_id = format!("*?{}?*", id);

    if !is_id {
        pat_id = format!("{}?*", id);
    }

    String::from(parent.join(pat_id).to_string_lossy())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[should_panic]
    fn check_reads_panic_test() {
        let input = PathBuf::from("./some_seq_reads.fastq.gz");
        let id = "ABC1234";
        let reads = vec![input];
        check_reads(&reads, &id);
    }

    #[test]
    #[should_panic(
        expected = "CANNOT FIND FILE ABC1234. USE THE --id FLAG IF YOU USE THE FILE ID."
    )]
    fn check_reads_panic_msg_test() {
        let id = "ABC1234";
        let reads = Vec::new();
        check_reads(&reads, &id);
    }

    #[test]
    #[should_panic]
    fn check_multireads_panic_test() {
        let input_1 = PathBuf::from("./some_seq_read1.fastq.gz");
        let input_2 = PathBuf::from("./some_seq_read1_l1.fastq.gz");
        let input_3 = PathBuf::from("./some_seq_read2.fastq.gz");
        let id = "ABC1234";
        let reads = vec![input_1, input_2, input_3];
        check_reads(&reads, &id);
    }

    #[test]
    fn glob_raw_reads_test() {
        let input = PathBuf::from("test_files/data.test");
        let pattern = "cde";

        let files = glob_raw_reads(&input, &pattern, true);

        assert_eq!(2, files.len());
    }

    #[test]
    fn glob_id_at_start_test() {
        let input = PathBuf::from("test_files/data.test");
        let pattern = "test_1";
        let is_id = false;

        let files = glob_raw_reads(&input, &pattern, is_id);

        assert_eq!(2, files.len());
    }
    #[test]
    fn parse_csv_test() {
        let input = PathBuf::from("test_files/test.csv");

        let seq = parse_csv(&input, true, false);

        assert_eq!(1, seq.len());

        seq.iter().for_each(|s| {
            let dir = input.parent().unwrap();
            assert_eq!(dir.join("test_1_cde_R1.fastq"), s.read_1);
            assert_eq!(dir.join("test_1_cde_R2.fastq"), s.read_2);
            assert_eq!("AGTCT", s.adapter_i5.as_ref().unwrap());
        });
    }

    #[test]
    fn parse_csv_pattern_test() {
        let input = PathBuf::from("test_files/test2.csv");

        let seq = parse_csv(&input, true, false);

        seq.iter().for_each(|s| {
            let dir = input.parent().unwrap();
            assert_eq!(dir.join("some_animals_XYZ12345_R1.fastq.gz"), s.read_1);
            assert_eq!(dir.join("some_animals_XYZ12345_R2.fastq.gz"), s.read_2);
            assert_eq!("ATGTCTCTCTATATATACT", s.adapter_i5.as_ref().unwrap());
        });
    }

    #[test]
    fn parse_csv_dual_indexes_test() {
        let input = PathBuf::from("test_files/dual_index_test.csv");

        let seq = parse_csv(&input, true, false);
        let i5 = "ATGTCTCTCTATATATACT";
        let i7 = String::from("ATGTCTCTCTATATATGCT");
        seq.iter().for_each(|s| {
            let dir = input.parent().unwrap();
            assert_eq!(dir.join("some_animals_XYZ12345_R1.fastq.gz"), s.read_1);
            assert_eq!(dir.join("some_animals_XYZ12345_R2.fastq.gz"), s.read_2);
            assert_eq!(i5, s.adapter_i5.as_ref().unwrap());
            assert_eq!(true, s.adapter_i7.is_some());
            assert_eq!(i7, String::from(s.adapter_i7.as_ref().unwrap()))
        });
    }

    #[test]
    #[should_panic]
    fn parse_csv_panic_test() {
        let input = PathBuf::from("test_files/invalid.csv");

        parse_csv(&input, true, false);
    }

    #[test]
    #[should_panic]
    fn parse_csv_multicols_panic_test() {
        let input = PathBuf::from("test_files/invalid_multicols.csv");

        parse_csv(&input, true, false);
    }

    #[test]
    fn get_adapter_test() {
        let mut seq = RawSeq::new();
        let id = String::from("MNCT");
        let i5 = String::from("ATGTGTGTGATatc");
        let i7 = String::from("ATTTGTGTTTCCC");

        let adapters: Vec<String> = vec![id, i5, i7];

        get_adapters(&mut seq, &adapters);

        assert_eq!("ATGTGTGTGATATC", seq.adapter_i5.as_ref().unwrap());
    }

    #[test]
    fn get_adapter_insert_test() {
        let mut seq = RawSeq::new();
        let id = String::from("MNCT");
        let i5 = String::from("ATGTGTGTGA*Tatc");
        let i7 = String::from("ATTTGTGTTT*CCC");

        let tag_i5 = String::from("ATT");
        let tag_i7 = String::from("GCC");

        let adapters: Vec<String> = vec![id, i5, i7, tag_i5, tag_i7];

        get_adapters(&mut seq, &adapters);

        assert_eq!("ATGTGTGTGATAATATC", seq.adapter_i5.as_ref().unwrap());
        assert_eq!(
            "ATTTGTGTTTCGGCCC",
            String::from(seq.adapter_i7.as_ref().unwrap())
        );
    }

    #[test]

    fn is_insert_test() {
        let seq = "ATATTAT*T";

        assert_eq!(true, is_insert_missing(seq));
    }

    #[test]
    fn target_dir_name_test() {
        let input = PathBuf::from("test_files/test_rename.csv");
        let is_rename = true;
        let is_id = false;

        let reads = parse_csv(&input, is_id, is_rename);

        reads.iter().for_each(|r| {
            let res = PathBuf::from("Rattus_rattus_XYZ12345");
            let id = String::from("some_animals_XYZ12345");
            assert_eq!(id, r.id);
            assert_eq!(res, r.dir);
            assert_eq!(true, r.auto_idx);
        });
    }
}
