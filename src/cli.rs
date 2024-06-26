use std::path::PathBuf;

use clap::{App, AppSettings, Arg, ArgMatches};

use crate::assembly;
use crate::assembly::cleaner;
use crate::checker;
use crate::init::Init;
use crate::qc;
use crate::stats;

fn get_args(version: &str) -> ArgMatches {
    App::new("YAP")
        .version(version)
        .about("A cli app for phylogenomics")
        .author("Heru Handika <hhandi1@lsu.edu>")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(App::new("check").about("Checks dependencies"))
        .subcommand(
            App::new("new")
                .about("Find sequences and generate input files")
                .arg(
                    Arg::with_name("dir")
                        .short("d")
                        .long("dir")
                        .help("Specify input directory")
                        .takes_value(true)
                        .default_value("raw_reads")
                        .value_name("DIR"),
                )
                .arg(
                    Arg::with_name("len")
                        .short("l")
                        .long("len")
                        .help("Word lengths")
                        .takes_value(true)
                        .default_value("3")
                        .value_name("LEN"),
                )
                .arg(
                    Arg::with_name("sep")
                        .short("s")
                        .long("sep")
                        .help("Separator type")
                        .takes_value(true)
                        .default_value("_")
                        .value_name("SEP"),
                )
                .arg(
                    Arg::with_name("csv")
                        .long("csv")
                        .takes_value(false)
                        .help("Save as csv"),
                ),
        )
        .subcommand(
            App::new("qc")
                .about("Trims adapters and clean low quality reads using fastp")
                .arg(
                    Arg::with_name("input")
                        .short("i")
                        .long("input")
                        .help("Inputs a config file")
                        .takes_value(true)
                        .default_value("yap-qc_input.conf")
                        .value_name("INPUT"),
                )
                .arg(
                    Arg::with_name("id")
                        .long("id")
                        .help("Uses id instead of filenames")
                        .takes_value(false),
                )
                .arg(
                    Arg::with_name("dryrun")
                        .long("dry")
                        .help("Checks if the program detect the correct files")
                        .takes_value(false),
                )
                .arg(
                    Arg::with_name("rename")
                        .long("rename")
                        .help("Renames output files")
                        .takes_value(false),
                )
                .arg(
                    Arg::with_name("output")
                        .short("o")
                        .long("output")
                        .help("Specifies output folders")
                        .takes_value(true)
                        .value_name("OUTPUT DIR"),
                )
                .arg(
                    Arg::with_name("opts")
                        .long("opts")
                        .help("Sets optional Fastp params")
                        .require_equals(true)
                        .takes_value(true)
                        .value_name("OPTIONAL PARAMS"),
                ),
        )
        .subcommand(
            App::new("assembly")
                .about("Assemble reads using SPAdes")
                .subcommand(
                    App::new("auto")
                        .about("Auto find clean reads and assembly them")
                        .arg(
                            Arg::with_name("dir")
                                .short("d")
                                .long("dir")
                                .help("Inputs a directory for auto search")
                                .takes_value(true)
                                .value_name("CLEAN-READ DIR")
                                .default_value("clean_reads"),
                        )
                        .arg(
                            Arg::with_name("specify")
                                .short("s")
                                .long("specify")
                                .help("Specifies clean read directory names")
                                .takes_value(true)
                                .default_value("trimmed")
                                .value_name("DIR NAME"),
                        )
                        .arg(
                            Arg::with_name("output")
                                .short("o")
                                .long("output")
                                .help("Specifies output folders")
                                .takes_value(true)
                                .value_name("OUTPUT DIR"),
                        )
                        .arg(
                            Arg::with_name("dryrun")
                                .long("dry")
                                .help("Checks if the program can find the correct files")
                                .takes_value(false),
                        )
                        .arg(
                            Arg::with_name("threads")
                                .short("t")
                                .long("threads")
                                .help("Sets number of threads")
                                .takes_value(true)
                                .value_name("THREAD-NUM"),
                        )
                        .arg(
                            Arg::with_name("opts")
                                .long("opts")
                                .help("Sets optional SPAdes params")
                                .require_equals(true)
                                .takes_value(true)
                                .value_name("OPTIONAL PARAMS"),
                        ),
                )
                .subcommand(
                    App::new("conf")
                        .about("Runs SPAdes using a config file")
                        .arg(
                            Arg::with_name("input")
                                .short("i")
                                .long("input")
                                .help("Inputs a config file")
                                .takes_value(true)
                                .value_name("INPUT"),
                        )
                        .arg(
                            Arg::with_name("dryrun")
                                .long("dry")
                                .help("Checks if the program detect the correct files")
                                .takes_value(false),
                        )
                        .arg(
                            Arg::with_name("threads")
                                .short("t")
                                .long("threads")
                                .help("Sets number of threads")
                                .takes_value(true)
                                .value_name("THREAD-NUM"),
                        )
                        .arg(
                            Arg::with_name("output")
                                .short("o")
                                .long("output")
                                .help("Specifies output folders")
                                .takes_value(true)
                                .value_name("OUTPUT DIR"),
                        )
                        .arg(
                            Arg::with_name("opts")
                                .long("opts")
                                .help("Sets optional SPAdes params")
                                .takes_value(true)
                                .value_name("OPTIONAL PARAMS"),
                        ),
                )
                .subcommand(
                    App::new("clean").about("Cleans unused SPAdes files.").arg(
                        Arg::with_name("dir")
                            .short("d")
                            .long("dir")
                            .help("Inputs a directory for cleaning")
                            .takes_value(true)
                            .value_name("DIR")
                            .required(true),
                    ),
                ),
        )
        .subcommand(
            App::new("stats")
                .about("Get sequence statistics")
                .arg(
                    Arg::with_name("wildcard")
                        .short("c")
                        .long("wcard")
                        .help("Finds files using wildcards. Allows multiple inputs")
                        .conflicts_with_all(&["dir", "file", "wdir"])
                        .multiple(true)
                        .value_name("WILDCARD"),
                )
                .arg(
                    Arg::with_name("wdir")
                        .short("w")
                        .long("walk")
                        .help("Tranverses through nested directories")
                        .conflicts_with_all(&["dir", "file", "wildcard"])
                        .takes_value(true)
                        .value_name("PARENT DIR"),
                )
                .arg(
                    Arg::with_name("format")
                        .short("f")
                        .long("format")
                        .help("Specifies input format")
                        .required(true)
                        .takes_value(true)
                        .possible_values(&["fastq", "fasta"])
                        .default_value("fastq")
                        .value_name("PARENT DIR"),
                )
                .arg(
                    Arg::with_name("nocsv")
                        .long("nocsv")
                        .help("Does not save results")
                        .takes_value(false),
                ),
        )
        .get_matches()
}

pub fn parse_cli(version: &str) {
    let args = get_args(version);
    match args.subcommand() {
        ("new", Some(init_matches)) => new_input(init_matches),
        ("assembly", Some(assembly_matches)) => match_assembly_cli(assembly_matches, version),
        ("qc", Some(qc_matches)) => Fastp::match_cli(qc_matches, version),
        ("check", Some(_)) => checker::check_dependencies().unwrap(),
        ("stats", Some(stats_matches)) => match_stats_cli(stats_matches, version),
        _ => unreachable!(),
    };
}

fn new_input(matches: &ArgMatches) {
    let path = matches.value_of("dir").expect("IS NOT A VALID FILE PATH");
    let len = matches
        .value_of("len")
        .unwrap()
        .parse::<usize>()
        .expect("NOT AN INTEGER");
    let sep = matches
        .value_of("sep")
        .unwrap()
        .parse::<char>()
        .expect("SEPARATOR SHOULD BE A SINGLE CHARACTER");
    let iscsv = matches.is_present("csv");
    let mut init = Init::new(path, len, sep, iscsv);

    init.initialize_input_file();
}

fn match_assembly_cli(args: &ArgMatches, version: &str) {
    let mut spades = Spades::new();
    println!("Starting YAP-assembly v{}...\n", version);
    match args.subcommand() {
        ("auto", Some(clean_matches)) => spades.run_auto(clean_matches),
        ("conf", Some(assembly_matches)) => spades.run(assembly_matches),
        ("clean", Some(clean_matches)) => spades.clean_files(clean_matches),
        _ => unreachable!(),
    };
}

fn match_stats_cli(args: &ArgMatches, version: &str) {
    let mut stats = Stats::new();
    println!("Starting YAP-stats v{}...\n", version);
    let value = args.value_of("format").expect("IS NOT A VALID FILE PATH");
    match value {
        "fastq" => stats.match_fastq(args),
        "fasta" => stats.match_fasta(args),
        _ => unreachable!("Please specify the allowed values"),
    }
}

trait Opts {
    fn get_params(&self, matches: &ArgMatches) -> Option<String> {
        let mut params = None;
        if matches.is_present("opts") {
            let input = matches.value_of("opts").unwrap();
            params = Some(String::from(input));
        }
        params
    }

    fn get_outdir(&mut self, matches: &ArgMatches) -> Option<PathBuf> {
        let mut outdir = None;
        if matches.is_present("output") {
            outdir = Some(PathBuf::from(matches.value_of("output").unwrap()));
        }
        outdir
    }
}

impl Opts for Fastp<'_> {}
impl Opts for Spades {}

struct Fastp<'a> {
    outdir: Option<PathBuf>,
    version: &'a str,
    matches: &'a ArgMatches<'a>,
}

impl<'a> Fastp<'a> {
    fn match_cli(matches: &'a ArgMatches, version: &'a str) {
        let mut set = Self {
            outdir: None,
            matches,
            version,
        };
        set.run();
    }

    fn run(&mut self) {
        if self.matches.is_present("input") {
            let path = PathBuf::from(self.matches.value_of("input").unwrap());
            let is_id = self.matches.is_present("id");
            let is_rename = self.matches.is_present("rename");
            let opts = self.get_params(&self.matches);
            self.outdir = self.get_outdir(self.matches);

            if self.matches.is_present("dryrun") {
                qc::dry_run(&path, is_id, is_rename);
            } else {
                println!("Starting YAP-qc v{}...\n", self.version);
                qc::process_input(&path, is_id, is_rename, &opts, &self.outdir);
            }
        }
    }
}

struct Spades {
    outdir: Option<PathBuf>,
}

impl Spades {
    fn new() -> Self {
        Self { outdir: None }
    }

    fn run_auto(&mut self, matches: &ArgMatches) {
        let path = matches.value_of("dir").unwrap();
        let dirname = matches.value_of("specify").unwrap();
        let threads = self.get_thread_num(matches);
        self.get_outdir(matches);
        let args = self.get_params(matches);
        if matches.is_present("dryrun") {
            assembly::auto_dryrun(path, &dirname)
        } else {
            assembly::auto_process_input(path, dirname, &threads, &self.outdir, &args);
        }
    }

    fn run(&mut self, matches: &ArgMatches) {
        let path = matches.value_of("input").unwrap();
        let threads = self.get_thread_num(matches);
        self.outdir = self.get_outdir(matches);
        let args = self.get_params(matches);
        if matches.is_present("dryrun") {
            assembly::dryrun(path)
        } else {
            assembly::process_input(path, &threads, &self.outdir, &args);
        }
    }

    fn clean_files(&self, matches: &ArgMatches) {
        let path = PathBuf::from(matches.value_of("dir").unwrap());
        cleaner::clean_spades_files(&path);
    }

    fn get_thread_num(&self, matches: &ArgMatches) -> Option<usize> {
        let mut threads = None;
        if matches.is_present("threads") {
            let num = matches.value_of("threads");
            match num {
                Some(n) => threads = Some(n.parse::<usize>().unwrap()),
                None => panic!("INVALID THREAD NUMBERS!"),
            }
        }
        threads
    }
}

struct Stats {
    fastq: bool,
    is_csv: bool,
}

impl Stats {
    fn new() -> Self {
        Self {
            fastq: false,
            is_csv: true,
        }
    }

    fn match_fastq(&mut self, matches: &ArgMatches) {
        self.check_is_nocsv(matches);
        self.fastq = true;
        self.get_stats(matches);
    }

    fn match_fasta(&mut self, matches: &ArgMatches) {
        self.check_is_nocsv(matches);
        self.get_stats(matches);
    }

    fn get_stats(&self, matches: &ArgMatches) {
        if matches.is_present("wildcard") {
            self.get_stats_wildcard(matches);
        } else if matches.is_present("wdir") {
            self.get_stats_walkdir(matches);
        }
    }

    fn get_stats_wildcard(&self, matches: &ArgMatches) {
        let entries: Vec<&str> = matches.values_of("wildcard").unwrap().collect();
        stats::process_wildcard(&entries, self.is_csv, self.fastq)
    }

    fn get_stats_walkdir(&self, matches: &ArgMatches) {
        let entry = matches.value_of("wdir").unwrap();
        stats::process_walkdir(&entry, self.is_csv, self.fastq);
    }

    fn check_is_nocsv(&mut self, matches: &ArgMatches) {
        if matches.is_present("nocsv") {
            self.is_csv = false;
        }
    }
}
