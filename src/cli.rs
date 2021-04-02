use std::path::PathBuf;

use clap::{App, AppSettings, Arg, ArgMatches};

use crate::assembly::asm_io;
use crate::assembly::cleaner;
use crate::checker;
use crate::qc::qc_io;

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
        .get_matches()
}

pub fn parse_cli(version: &str) {
    let args = get_args(version);
    match args.subcommand() {
        ("assembly", Some(assembly_matches)) => match_assembly_cli(assembly_matches, version),
        ("qc", Some(clean_matches)) => run_fastp_clean(clean_matches, version),
        ("check", Some(_)) => checker::check_dependencies().unwrap(),
        _ => unreachable!(),
    };
}

fn match_assembly_cli(args: &ArgMatches, version: &str) {
    let spades = Spades::new(version);
    match args.subcommand() {
        ("auto", Some(clean_matches)) => spades.run_spades_auto(clean_matches),
        ("conf", Some(assembly_matches)) => spades.run_spades(assembly_matches),
        ("clean", Some(clean_matches)) => spades.clean_spades_files(clean_matches),
        _ => (),
    };
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
            println!("Starting fastp-runner v{}...\n", version);
            qc_io::process_input(&path, is_id, is_rename, &opts);
        }
    }
}

struct Spades<'a> {
    version: &'a str,
}

impl<'a> Spades<'a> {
    fn new(version: &'a str) -> Self {
        Self { version }
    }

    fn run_spades_auto(&self, matches: &ArgMatches) {
        let path = matches.value_of("dir").unwrap();
        let dirname = matches.value_of("specify").unwrap();
        let threads = self.get_thread_num(matches);
        let outdir = self.get_outdir(matches);
        let args = get_opts(matches);
        if matches.is_present("dryrun") {
            asm_io::auto_dryrun(path, &dirname)
        } else {
            println!("Starting spade-runner v{}...\n", self.version);
            asm_io::auto_process_input(path, dirname, &threads, &outdir, &args);
        }
    }

    fn run_spades(&self, matches: &ArgMatches) {
        let path = matches.value_of("input").unwrap();
        let threads = self.get_thread_num(matches);
        let outdir = self.get_outdir(matches);
        let args = get_opts(matches);
        if matches.is_present("dryrun") {
            asm_io::dryrun(path)
        } else {
            println!("Starting spade-runner v{}...\n", self.version);
            asm_io::process_input(path, &threads, &outdir, &args);
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

    fn get_outdir(&self, matches: &ArgMatches) -> Option<PathBuf> {
        let mut dir = None;
        if matches.is_present("output") {
            dir = Some(PathBuf::from(matches.value_of("output").unwrap()));
        }
        dir
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
