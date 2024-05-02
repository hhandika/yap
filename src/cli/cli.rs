use std::io::Result;

use clap::crate_version;
use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;

use crate::assembly;
use crate::assembly::cleaner;
use crate::checker::DependencyChecker;
use crate::cli::args;
use crate::init::Init;
use crate::qc::Qc;
use clap::Parser;

use super::args::{AssemblySubcommand, AutoArgs, CleanArgs, ConfArgs, NewSubcommand, QcSubcommand};

pub const LOG_FILE: &str = "yap.log";

pub fn parse_cli() {
    let args = args::Cli::parse();
    let version = crate_version!();
    setup_logger().expect("Failed setting up logger");
    match args.subcommand {
        args::MainSubcommand::Check(arg) => DependencyChecker::new(arg.auto_install).check(),
        args::MainSubcommand::New(new) => parse_new_cli(&new),
        args::MainSubcommand::Qc(qc) => QcCli::new(&qc, version).parse(),
        args::MainSubcommand::Assembly(assembly) => Spades::new(&assembly, version).parse(),
    };
}

fn parse_new_cli(command: &NewSubcommand) {
    let path = command.dir.as_path();
    let len = command.len;
    let sep = command.sep;
    let is_csv = command.csv;
    let mut init = Init::new(path, len, sep, is_csv);

    init.initialize_input_file();
}

struct QcCli<'a> {
    version: &'a str,
    matches: &'a QcSubcommand,
}

impl<'a> QcCli<'a> {
    fn new(matches: &'a QcSubcommand, version: &'a str) -> Self {
        Self { version, matches }
    }

    fn parse(&self) {
        let input_path = self.matches.input.as_path();
        let is_rename = self.matches.rename;
        let optional_params = self.matches.opts.as_deref();
        let output_dir = self.matches.output.as_deref();
        let is_dry_run = self.matches.dry_run;

        let runner = Qc::new(input_path, is_rename, optional_params, output_dir);

        if is_dry_run {
            runner.dry_run();
        } else {
            log::info!("Starting YAP-qc v{}...\n", self.version);
            runner.run();
        }
    }
}

struct Spades<'a> {
    version: &'a str,
    matches: &'a AssemblySubcommand,
}

impl<'a> Spades<'a> {
    fn new(matches: &'a AssemblySubcommand, version: &'a str) -> Self {
        Self { version, matches }
    }

    fn parse(&self) {
        match self.matches {
            AssemblySubcommand::Auto(auto) => self.run_auto(auto),
            AssemblySubcommand::Conf(conf) => self.run(conf),
            AssemblySubcommand::Clean(clean) => self.clean_files(clean),
        }
    }

    fn run_auto(&self, matches: &AutoArgs) {
        let input_dir = matches.dir.as_path();
        let output = matches.output.as_deref();
        let optional_params = matches.opts.as_deref();
        let threads = matches.threads;
        let dry_run = matches.dry_run;
        if dry_run {
            assembly::auto_dry_run(input_dir, &matches.specify);
        } else {
            self.print_header();
            assembly::auto_process_input(
                input_dir,
                &matches.specify,
                threads,
                output,
                optional_params,
            );
        }
    }

    fn run(&self, matches: &ConfArgs) {
        let config = matches.input.as_path();
        let threads = matches.threads;
        let output = matches.output.as_deref();
        let dry_run = matches.dry_run;
        if dry_run {
            assembly::dry_run(config);
        } else {
            self.print_header();
            assembly::process_input(config, threads, output, None);
        }
    }

    fn clean_files(&self, matches: &CleanArgs) {
        let dir = matches.dir.as_path();
        cleaner::clean_spades_files(&dir);
    }

    fn print_header(&self) {
        log::info!("Starting YAP-assembly v{}...", self.version);
    }
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
