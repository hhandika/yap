use std::path::PathBuf;

use clap::{App, AppSettings, Arg, ArgMatches};

use crate::assembly::asm_io;
use crate::assembly::cleaner;
use crate::checker;
use crate::qc::qc_io;
use crate::stats::input;

fn get_args(version: &str) -> ArgMatches {
    App::new("YAP")
        .version(version)
        .about("A cli app for phylogenomics")
        .author("Heru Handika <hhandi1@lsu.edu>")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(App::new("check").about("Checks dependencies"))
        .subcommand(
            App::new("qc")
                .about("Trims adapters and clean low quality reads using fastp")
                .arg(
                    Arg::with_name("input")
                        .short("i")
                        .long("input")
                        .help("Inputs a config file")
                        .takes_value(true)
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
                    Arg::with_name("opts")
                        .long("opts")
                        .help("Sets optional SPAdes params")
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
                                .required(true),
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
                .subcommand(
                    App::new("fastq")
                        .about("Uses for FASTQ (raw-sequences) inputs")
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
                            Arg::with_name("nocsv")
                                .long("nocsv")
                                .help("Does not save results")
                                .takes_value(false),
                        ),
                )
                .subcommand(
                    App::new("fasta")
                        .about("Uses for FASTA (sequence assemblies) inputs")
                        .arg(
                            Arg::with_name("wildcard")
                                .short("c")
                                .long("wcard")
                                .help("Finds files using wildcards. Allows multiple inputs")
                                .conflicts_with_all(&["dir", "file", "wdir"])
                                .multiple(true)
                                .value_name("WILDCARDS"),
                        )
                        .arg(
                            Arg::with_name("wdir")
                                .short("w")
                                .long("walk")
                                .help("Tranverses nested directories")
                                .conflicts_with_all(&["dir", "file", "wildcard"])
                                .takes_value(true)
                                .value_name("PARENT DIR"),
                        )
                        .arg(
                            Arg::with_name("nocsv")
                                .long("nocsv")
                                .help("Does not save results")
                                .takes_value(false),
                        ),
                ),
        )
        .get_matches()
}

pub fn parse_cli(version: &str) {
    let args = get_args(version);
    match args.subcommand() {
        ("assembly", Some(assembly_matches)) => match_assembly_cli(assembly_matches, version),
        ("qc", Some(qc_matches)) => run_fastp_clean(qc_matches, version),
        ("check", Some(_)) => checker::check_dependencies().unwrap(),
        ("stats", Some(stats_matches)) => match_stats_cli(stats_matches, version),
        _ => unreachable!(),
    };
}

fn match_assembly_cli(args: &ArgMatches, version: &str) {
    let mut spades = Spades::new();
    println!("Starting YAP-assembly v{}...\n", version);
    match args.subcommand() {
        ("auto", Some(clean_matches)) => spades.run_spades_auto(clean_matches),
        ("conf", Some(assembly_matches)) => spades.run_spades(assembly_matches),
        ("clean", Some(clean_matches)) => spades.clean_spades_files(clean_matches),
        _ => unreachable!(),
    };
}

fn match_stats_cli(args: &ArgMatches, version: &str) {
    let mut stats = Stats::new();
    println!("Starting YAP-stats v{}...\n", version);
    match args.subcommand() {
        ("fastq", Some(fastq_matches)) => stats.match_fastq(fastq_matches),
        ("fasta", Some(fasta_matches)) => stats.match_fasta(fasta_matches),
        _ => unreachable!(),
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
        if matches.is_present("nocsv") {
            self.is_csv = false;
        }
        self.fastq = true;
        self.get_stats(matches);
    }

    fn match_fasta(&mut self, matches: &ArgMatches) {
        self.is_csv = matches.is_present("nocsv");
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
        input::process_wildcard(&entries, self.is_csv, self.fastq)
    }

    fn get_stats_walkdir(&self, matches: &ArgMatches) {
        let is_csv = matches.is_present("nocsv");
        let entry = matches.value_of("wdir").unwrap();
        input::process_walkdir(&entry, is_csv, true);
    }
}

fn run_fastp_clean(matches: &ArgMatches, version: &str) {
    if matches.is_present("input") {
        let path = PathBuf::from(matches.value_of("input").unwrap());
        let is_id = matches.is_present("id");
        let is_rename = matches.is_present("rename");

        let opts = get_opts(&matches);

        if matches.is_present("dryrun") {
            qc_io::dry_run(&path, is_id, is_rename);
        } else {
            println!("Starting YAP-qc v{}...\n", version);
            qc_io::process_input(&path, is_id, is_rename, &opts);
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

    fn run_spades_auto(&mut self, matches: &ArgMatches) {
        let path = matches.value_of("dir").unwrap();
        let dirname = matches.value_of("specify").unwrap();
        let threads = self.get_thread_num(matches);
        self.get_outdir(matches);
        let args = get_opts(matches);
        if matches.is_present("dryrun") {
            asm_io::auto_dryrun(path, &dirname)
        } else {
            asm_io::auto_process_input(path, dirname, &threads, &self.outdir, &args);
        }
    }

    fn run_spades(&mut self, matches: &ArgMatches) {
        let path = matches.value_of("input").unwrap();
        let threads = self.get_thread_num(matches);
        self.get_outdir(matches);
        let args = get_opts(matches);
        if matches.is_present("dryrun") {
            asm_io::dryrun(path)
        } else {
            asm_io::process_input(path, &threads, &self.outdir, &args);
        }
    }

    fn clean_spades_files(&self, matches: &ArgMatches) {
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

    fn get_outdir(&mut self, matches: &ArgMatches) {
        if matches.is_present("output") {
            self.outdir = Some(PathBuf::from(matches.value_of("output").unwrap()));
        }
    }
}

fn get_opts(matches: &ArgMatches) -> Option<String> {
    let mut opts = None;
    if matches.is_present("opts") {
        let input = matches.value_of("opts").unwrap();
        let params = input.replace("params=", "");
        opts = Some(String::from(params.trim()));
    }
    opts
}
