use std::path::PathBuf;

use clap::{App, AppSettings, Arg, ArgMatches};

use crate::assembly::cleaner;
use crate::assembly::io;
use crate::checker;

fn get_args(version: &str) -> ArgMatches {
    App::new("YAP")
        .version(version)
        .about("A pipeline for processing sequence capture data.")
        .author("Heru Handika <hhandi1@lsu.edu>")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(App::new("check").about("Checks dependencies"))
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
                            Arg::with_name("dry-run")
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
                            Arg::with_name("dry-run")
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

pub fn get_cli(version: &str) {
    let args = get_args(version);
    match args.subcommand() {
        ("assembly", Some(assemble_matches)) => match_assembly_cli(assemble_matches, version),
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
        let dir = self.get_dir(matches);
        let args = self.get_args(matches);
        if matches.is_present("dry-run") {
            io::auto_dryrun(path, &dirname)
        } else {
            println!("Starting spade-runner v{}...\n", self.version);
            io::auto_process_input(path, &dirname, &threads, &dir, &args);
        }
    }

    fn run_spades(&self, matches: &ArgMatches) {
        let path = matches.value_of("input").unwrap();
        let threads = self.get_thread_num(matches);
        let dir = self.get_dir(matches);
        let args = self.get_args(matches);
        if matches.is_present("dry-run") {
            io::dryrun(path)
        } else {
            println!("Starting spade-runner v{}...\n", self.version);
            io::process_input(path, &threads, &dir, &args);
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

    fn get_dir(&self, matches: &ArgMatches) -> Option<PathBuf> {
        let mut dir = None;
        if matches.is_present("output") {
            dir = Some(PathBuf::from(matches.value_of("output").unwrap()));
        }
        dir
    }

    fn get_args(&self, matches: &ArgMatches) -> Option<String> {
        let mut dir = None;
        if matches.is_present("opts") {
            let input = matches.value_of("opts").unwrap();
            let args = input.replace("params=", "");
            dir = Some(String::from(args.trim()));
        }
        dir
    }
}
