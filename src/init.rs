use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{LineWriter, Write};

use regex::Regex;
use walkdir::WalkDir;

pub struct Init<'a> {
    path: &'a str,
    len: usize,
    sep: char,
    iscsv: bool,
    fname: String,
}

impl<'a> Init<'a> {
    pub fn new(path: &'a str, len: usize, sep: char, iscsv: bool) -> Self {
        Self {
            path,
            len,
            sep,
            iscsv,
            fname: String::from("yap-qc_input"),
        }
    }

    pub fn initialize_input_file(&mut self) {
        self.get_fnames();
        let output = File::create(&self.fname).expect("FILE EXISTS.");
        let mut line = LineWriter::new(output);
        let seqs = self.find_files();
        self.write_header(&mut line);
        seqs.iter().for_each(|(id, path)| {
            self.write_content(&mut line, &id, &path);
        });

        self.print_saved_path();
    }

    fn find_files(&self) -> HashMap<String, String> {
        let mut seq = HashMap::new();
        WalkDir::new(&self.path)
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
                    if !seq.contains_key(&id) {
                        seq.insert(id, full_path);
                    }
                }
            });

        seq
    }

    fn get_fnames(&mut self) {
        if self.iscsv {
            self.fname.push_str(".csv");
        } else {
            self.fname.push_str(".conf");
        }
    }

    fn write_header<W: Write>(&self, line: &mut W) {
        if self.iscsv {
            writeln!(line, "id,new_name").unwrap();
        } else {
            writeln!(line, "[seqs]").unwrap();
        }
    }

    fn write_content<W: Write>(&self, line: &mut W, id: &str, full_path: &str) {
        if self.iscsv {
            writeln!(line, "{}", id).unwrap();
        } else {
            writeln!(line, "{}:{}/", id, full_path).unwrap();
        }
    }

    fn print_saved_path(&self) {
        let path = env::current_dir().unwrap();
        println!(
            "Done! The result is saved as {}/{}",
            path.display(),
            self.fname
        );
    }

    fn re_matches_lazy(&self, fname: &str) -> bool {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"(_|-)((?i)(read|r)1)(?:.*)(gz|gzip)").unwrap();
        }

        RE.is_match(fname)
    }

    fn construct_id(&self, names: &str) -> String {
        let words: Vec<&str> = names.split(self.sep).collect();
        assert!(words.len() > self.len, "NO. OF WORDS EXCEED THE SLICES");
        let mut seqname = String::new();

        words[0..(self.len - 1)].iter().for_each(|w| {
            let comp = format!("{}{}", w, self.sep);
            seqname.push_str(&comp);
        });

        seqname.push_str(words[self.len - 1]);
        seqname
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn regex_test() {
        let path = "test_files/init/";
        let len = 3;
        let sep = '_';
        let re = Init::new(path, len, sep, true);
        let zipped_read = "sample_buno_clean_read1.fastq.gz";
        let unzipped_read = "sample_buno_clean_read1.fastq";

        assert_eq!(true, re.re_matches_lazy(zipped_read));
        assert_eq!(false, re.re_matches_lazy(unzipped_read));
    }

    #[test]
    fn construct_id_test() {
        let path = "test_files/init/";
        let len = 3;
        let sep = '_';
        let re = Init::new(path, len, sep, true);

        let fnames = "sample_buno_ABCD123_read1.fastq.gz";

        let id = re.construct_id(fnames);

        assert_eq!("sample_buno_ABCD123", id);
    }

    #[test]
    #[should_panic]
    fn construct_id_panic_test() {
        let path = "test_files/init/";
        let len = 4;
        let sep = '_';
        let re = Init::new(path, len, sep, true);
        let fnames = "sample_buno_ABCD123_read1.fastq.gz";

        re.construct_id(fnames);
    }
}
