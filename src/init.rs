use std::env;
use std::fs::File;
use std::io::{LineWriter, Write};

use regex::Regex;
use walkdir::WalkDir;

pub fn initialize_input_file(path: &str, len: usize, sep: char, iscsv: bool) {
    let save_names = get_fnames(iscsv);
    let output = File::create(&save_names).expect("FILE EXISTS.");
    let mut line = LineWriter::new(output);
    write_header(&mut line, iscsv);
    WalkDir::new(path)
        .into_iter()
        .filter_map(|ok| ok.ok())
        .filter(|e| e.file_type().is_file())
        .for_each(|e| {
            let path = e.path().parent().unwrap();
            let fname = e.path().file_name().unwrap().to_string_lossy();
            if re_matches_lazy(&fname) {
                let id = construct_id(&fname, len, sep);
                let full_path = String::from(path.canonicalize().unwrap().to_string_lossy());
                write_content(&mut line, &id, &full_path, iscsv);
            }
        });
    print_saved_path(&save_names);
}

fn write_header<W: Write>(line: &mut W, iscsv: bool) {
    if iscsv {
        writeln!(line, "id,path").unwrap();
    } else {
        writeln!(line, "[seq]").unwrap();
    }
}

fn write_content<W: Write>(line: &mut W, id: &str, full_path: &str, iscsv: bool) {
    if iscsv {
        writeln!(line, "{},{}/", id, full_path).unwrap();
    } else {
        writeln!(line, "{}:{}/", id, full_path).unwrap();
    }
}

fn print_saved_path(save_names: &str) {
    let path = env::current_dir().unwrap();
    println!(
        "Done! The result is saved as {}/{}",
        path.display(),
        save_names
    );
}
 
fn get_fnames(iscsv: bool) -> String {
    let mut fname = String::from("yap-qc_input");
    if iscsv {
        fname.push_str(".csv");
    } else {
        fname.push_str(".conf");
    }

    fname
}

fn re_matches_lazy(fname: &str) -> bool {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(_|-)((?i)(read|r)1)(?:.*)(gz|gzip)").unwrap();
    }

    RE.is_match(fname)
}

fn construct_id(names: &str, len: usize, sep: char) -> String {
    let words: Vec<&str> = names.split(sep).collect();
    assert!(words.len() > len, "NO. OF WORDS EXCEED THE SLICES");
    let mut seqname = String::new();

    words[0..(len - 1)].iter().for_each(|w| {
        let comp = format!("{}{}", w, sep);
        seqname.push_str(&comp);
    });

    seqname.push_str(words[len - 1]);
    seqname
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn regex_test() {
        let zipped_read = "sample_buno_clean_read1.fastq.gz";
        let unzipped_read = "sample_buno_clean_read1.fastq";

        assert_eq!(true, re_matches_lazy(zipped_read));
        assert_eq!(false, re_matches_lazy(unzipped_read));
    }

    #[test]
    fn regex_io_test() {
        use glob::glob;
        use std::path::PathBuf;

        let path = "test_files/*";
        let entries = glob(path)
            .unwrap()
            .filter_map(|ok| ok.ok())
            .collect::<Vec<PathBuf>>();

        let mut files = Vec::new();
        entries.iter().for_each(|e| {
            let path = String::from(e.file_name().unwrap().to_string_lossy());
            if re_matches_lazy(&path) {
                files.push(e);
            }
        });

        assert_eq!(3, files.len());
    }

    #[test]
    fn construct_id_test() {
        let fnames = "sample_buno_ABCD123_read1.fastq.gz";

        let id = construct_id(fnames, 3, '_');

        assert_eq!("sample_buno_ABCD123", id);
    }

    #[test]
    #[should_panic]
    fn construct_id_panic_test() {
        let fnames = "sample_buno_ABCD123_read1.fastq.gz";

        construct_id(fnames, 4, '_');
    }
}
