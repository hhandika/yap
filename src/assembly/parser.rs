use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;

pub fn parse_sequence_dir(input: &Path) -> Vec<SeqDirs> {
    let file = File::open(input).unwrap();
    let buff = BufReader::new(file);

    let mut sequence_dir = Vec::new();
    buff.lines().map_while(Result::ok).skip(1).for_each(|line| {
        let mut sample = SeqDirs::new();

        if line.contains(',') {
            sample.parse_csv(&line);
        } else if line.contains(':') {
            sample.parse_ini(&line);
        } else {
            panic!(
                "INVALID INPUT FORMAT. \
                    LOOKING FOR ',' or ':' FOUND {}",
                line
            );
        }
        sequence_dir.push(sample);
    });

    sequence_dir
}

pub struct SeqDirs {
    pub id: String,
    pub dir: String,
}

impl SeqDirs {
    fn new() -> Self {
        Self {
            id: String::new(),
            dir: String::new(),
        }
    }

    fn parse_csv(&mut self, line: &str) {
        let sep = ',';
        let lines = self.split_strings(line, sep);
        self.check_results(&lines);
        self.parse_samples(&lines);
    }

    fn parse_ini(&mut self, line: &str) {
        let sep = ':';
        let lines = self.split_strings(line, sep);
        self.check_results(&lines);
        self.parse_samples(&lines);
    }

    fn parse_samples(&mut self, lines: &[String]) {
        self.id = String::from(&lines[0]);
        self.dir = String::from(&lines[1]);
    }

    fn check_results(&self, lines: &[String]) {
        if lines.len() != 2 {
            panic!(
                "INVALID INPUT. EXPECTING ID AND DIRECTORY PATH, \
                FOUND: {:?}",
                lines
            )
        }
    }

    fn split_strings(&self, line: &str, sep: char) -> Vec<String> {
        line.split(sep).map(|e| e.trim().to_string()).collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn input_ini_test() {
        let input = Path::new("test_files/assembly/spade_runner.ini");
        let samples = parse_sequence_dir(&input);

        assert_eq!(2, samples.len());
    }

    #[test]
    fn split_csv_test() {
        let line = "some_animals,folder/target/";
        let mut samples = SeqDirs::new();

        samples.parse_csv(line);
        assert_eq!("some_animals", samples.id);
        assert_eq!("folder/target/", samples.dir);
    }

    #[test]
    fn split_csv_whitespaces_test() {
        let line = " some_animals,folder/target/ ";
        let mut samples = SeqDirs::new();

        samples.parse_csv(line);
        assert_eq!("some_animals", samples.id);
        assert_eq!("folder/target/", samples.dir);
    }

    #[test]
    fn split_ini_test() {
        let line = "some_animals:folder/target/";
        let mut samples = SeqDirs::new();

        samples.parse_ini(line);
        assert_eq!("some_animals", samples.id);
        assert_eq!("folder/target/", samples.dir);
    }

    #[test]
    #[should_panic]
    fn split_ini_panic_test() {
        let line = "some_animals:folder/target/:random";
        let mut samples = SeqDirs::new();

        samples.parse_ini(line);
    }
}
