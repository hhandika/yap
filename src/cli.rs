use std::io::Result;
use std::path::PathBuf;

use clap::{crate_description, crate_name, App, AppSettings, Arg, ArgMatches};
use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;

use crate::assembly;
use crate::assembly::cleaner;
use crate::checker;
use crate::init::Init;
use crate::qc;

const LOG_FILE: &str = "yap.log";

fn get_args(version: &str) -> ArgMatches {
    App::new(crate_name!())
        .version(version)
        .about(crate_description!())
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
        .get_matches()
}

pub fn parse_cli(version: &str) {
    let args = get_args(version);
    setup_logger().expect("Failed setting up logger");
    match args.subcommand() {
        ("new", Some(init_matches)) => new_input(init_matches),
        ("assembly", Some(assembly_matches)) => match_assembly_cli(assembly_matches, version),
        ("qc", Some(qc_matches)) => Fastp::match_cli(qc_matches, version),
        ("check", Some(_)) => checker::check_dependencies().unwrap(),
        _ => unreachable!(),
    };
}

fn setup_logger() -> Result<()> {
    let log_dir = std::env::current_dir()?;
    let target = log_dir.join(LOG_FILE);
    let tofile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S %Z)} - {l} - {m}\n",
        )))
        .build(target)?;

    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{m}\n")))
        .build();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("logfile", Box::new(tofile)))
        .build(
            Root::builder()
                .appender("stdout")
                .appender("logfile")
                .build(LevelFilter::Info),
        )
        .expect("Failed building log configuration");

    log4rs::init_config(config).expect("Cannot initiate log configuration");

    Ok(())
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
    let is_csv = matches.is_present("csv");
    let mut init = Init::new(path, len, sep, is_csv);

    init.initialize_input_file();
}

fn match_assembly_cli(args: &ArgMatches, version: &str) {
    let mut spades = Spades::new();
    log::info!("Starting YAP-assembly v{}...\n", version);
    match args.subcommand() {
        ("auto", Some(clean_matches)) => spades.run_auto(clean_matches),
        ("conf", Some(assembly_matches)) => spades.run(assembly_matches),
        ("clean", Some(clean_matches)) => spades.clean_files(clean_matches),
        _ => unreachable!(),
    };
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

    fn get_output_dir(&mut self, matches: &ArgMatches) -> Option<PathBuf> {
        let mut output_dir = None;
        if matches.is_present("output") {
            output_dir = Some(PathBuf::from(matches.value_of("output").unwrap()));
        }
        output_dir
    }
}

impl Opts for Fastp<'_> {}
impl Opts for Spades {}

struct Fastp<'a> {
    output_dir: Option<PathBuf>,
    version: &'a str,
    matches: &'a ArgMatches<'a>,
}

impl<'a> Fastp<'a> {
    fn match_cli(matches: &'a ArgMatches, version: &'a str) {
        let mut set = Self {
            output_dir: None,
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
            let opts = self.get_params(self.matches);
            self.output_dir = self.get_output_dir(self.matches);

            if self.matches.is_present("dryrun") {
                qc::dry_run(&path, is_id, is_rename);
            } else {
                log::info!("Starting YAP-qc v{}...\n", self.version);
                qc::process_input(&path, is_id, is_rename, &opts, &self.output_dir);
            }
        }
    }
}

struct Spades {
    output_dir: Option<PathBuf>,
}

impl Spades {
    fn new() -> Self {
        Self { output_dir: None }
    }

    fn run_auto(&mut self, matches: &ArgMatches) {
        let path = matches.value_of("dir").unwrap();
        let dirname = matches.value_of("specify").unwrap();
        let threads = self.get_thread_num(matches);
        self.get_output_dir(matches);
        let args = self.get_params(matches);
        if matches.is_present("dryrun") {
            assembly::auto_dry_run(path, dirname)
        } else {
            assembly::auto_process_input(path, dirname, &threads, &self.output_dir, &args);
        }
    }

    fn run(&mut self, matches: &ArgMatches) {
        let path = matches.value_of("input").unwrap();
        let threads = self.get_thread_num(matches);
        self.output_dir = self.get_output_dir(matches);
        let args = self.get_params(matches);
        if matches.is_present("dryrun") {
            assembly::dry_run(path)
        } else {
            assembly::process_input(path, &threads, &self.output_dir, &args);
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
