use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{LineWriter, Write};
use std::path::Path;

use regex::Regex;
use walkdir::WalkDir;

pub struct Init<'a> {
    path: &'a Path,
    len: usize,
    sep: char,
    is_csv: bool,
    fname: String,
}

impl<'a> Init<'a> {
    pub fn new(path: &'a Path, len: usize, sep: char, is_csv: bool) -> Self {
        Self {
            path,
            len,
            sep,
            is_csv,
            fname: String::from("yap-qc_input"),
        }
    }

    pub fn initialize_input_file(&mut self) {
        self.get_file_names();
        let output = File::create(&self.fname).expect("FILE EXISTS.");
        let mut line = LineWriter::new(output);
        let seqs = self.find_files();
        self.write_header(&mut line);
        let file_count = seqs.len();
        let mut sample_count = 0;
        seqs.iter().for_each(|(id, path)| {
            self.write_content(&mut line, id, path);
            sample_count += 1;
        });

        self.print_saved_path(file_count, sample_count);
    }

    fn find_files(&self) -> HashMap<String, String> {
        let mut seq = HashMap::new();
        WalkDir::new(self.path)
            .into_iter()
            .filter_map(|ok| ok.ok())
            .filter(|e| e.file_type().is_file())
            .for_each(|e| {
                let path = e.path();
                let fname = e.path().file_name().unwrap().to_string_lossy();
                if self.re_matches_lazy(&fname) {
                    let id = self.construct_id(&fname);
                    let full_path = String::from(
                        path.parent()
                            .unwrap()
                            .canonicalize()
                            .unwrap()
                            .to_string_lossy(),
                    );
                    seq.entry(id).or_insert(full_path);
                }
            });

        seq
    }

    fn get_file_names(&mut self) {
        if self.is_csv {
            self.fname.push_str(".csv");
        } else {
            self.fname.push_str(".conf");
        }
    }

    fn write_header<W: Write>(&self, line: &mut W) {
        if self.is_csv {
            writeln!(line, "id,new_name").unwrap();
        } else {
            writeln!(line, "[seqs]").unwrap();
        }
    }

    fn write_content<W: Write>(&self, line: &mut W, id: &str, full_path: &str) {
        if self.is_csv {
            writeln!(line, "{}", id).unwrap();
        } else {
            writeln!(line, "{}:{}/", id, full_path).unwrap();
        }
    }

    fn print_saved_path(&self, file_count: usize, sample_count: usize) {
        let path = env::current_dir().unwrap();
        println!(
            "Done! Found {} samples of {} files. \
            The result is saved as {}/{}",
            sample_count,
            file_count,
            path.display(),
            self.fname
        );
    }

    fn re_matches_lazy(&self, fname: &str) -> bool {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"(_|-)((?i)(read|r)1)(.fq|.fastq)(?:.*)").unwrap();
        }

        RE.is_match(fname)
    }

    fn construct_id(&self, names: &str) -> String {
        let words: Vec<&str> = names.split(self.sep).collect();
        assert!(words.len() > self.len, "NO. OF WORDS EXCEED THE SLICES");
        let mut sequence_name = String::new();

        words[0..(self.len - 1)].iter().for_each(|w| {
            let comp = format!("{}{}", w, self.sep);
            sequence_name.push_str(&comp);
        });

        sequence_name.push_str(words[self.len - 1]);
        sequence_name
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn regex_test() {
        let path = Path::new("test_files/init/");
        let len = 3;
        let sep = '_';
        let re = Init::new(path, len, sep, true);
        let zipped_read = "sample_buno_clean_read1.fastq.gz";
        let unzipped_read = "sample_buno_clean_read1.fastq";

        assert_eq!(true, re.re_matches_lazy(zipped_read));
        assert_eq!(true, re.re_matches_lazy(unzipped_read));
    }

    #[test]
    fn construct_id_test() {
        let path = Path::new("test_files/init/");
        let len = 3;
        let sep = '_';
        let re = Init::new(path, len, sep, true);

        let file_name = "sample_buno_ABCD123_read1.fastq.gz";

        let id = re.construct_id(file_name);

        assert_eq!("sample_buno_ABCD123", id);
    }

    #[test]
    #[should_panic]
    fn construct_id_panic_test() {
        let path = Path::new("test_files/init/");
        let len = 4;
        let sep = '_';
        let re = Init::new(path, len, sep, true);
        let file_name = "sample_buno_ABCD123_read1.fastq.gz";

        re.construct_id(file_name);
    }
}
