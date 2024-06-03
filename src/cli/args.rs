use std::path::PathBuf;

use clap::{crate_authors, crate_description, crate_name, crate_version, Args, Parser, Subcommand};

use crate::cli;

#[derive(Parser)]
#[command(name = crate_name!())]
#[command(version = crate_version!())]
#[command(author = crate_authors!())]
#[command(about = crate_description!(), long_about = None)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) subcommand: MainSubcommand,
    #[arg(
        long = "log",
        help = "Log file path",
        default_value = cli::cli::LOG_FILE,
        global = true
    )]
    pub(crate) log: PathBuf,
}

#[derive(Subcommand)]
pub(crate) enum MainSubcommand {
    #[command(name = "check", about = "Checks dependencies")]
    Check(Check),
    #[command(
        name = "new",
        about = "Find sequences and generate input files",
        long_about = None
    )]
    New(NewSubcommand),
    #[command(
        name = "qc",
        about = "Trims adapters and clean low quality reads using fastp",
        long_about = None
    )]
    Qc(QcSubcommand),
    #[command(
        subcommand,
        name = "assembly",
        about = "Assemble reads using SPAdes",
        long_about = None
    )]
    Assembly(AssemblySubcommand),
}

#[derive(Args)]
pub(crate) struct Check {
    #[arg(long, help = "Auto install missing dependencies")]
    pub(crate) auto_install: bool,
}

#[derive(Args)]
pub(crate) struct NewSubcommand {
    #[arg(short, long, default_value = "raw_reads", value_name = "DIR")]
    pub(crate) dir: PathBuf,
    #[arg(short, long, default_value = "3", value_name = "LEN")]
    pub(crate) len: usize,
    #[arg(short, long, default_value = "_", value_name = "SEP")]
    pub(crate) sep: char,
    #[arg(long, help = "Save as csv")]
    pub(crate) csv: bool,
}

#[derive(Args)]
pub(crate) struct QcSubcommand {
    #[arg(short, long, default_value = "yap-qc_input.conf", value_name = "INPUT")]
    pub(crate) input: PathBuf,
    #[arg(long, help = "Checks if the program detect the correct files")]
    pub(crate) dry_run: bool,
    #[arg(long, help = "Renames output files")]
    pub(crate) rename: bool,
    #[arg(short, long, value_name = "OUTPUT DIR")]
    pub(crate) output: Option<PathBuf>,
    #[arg(long, value_name = "OPTIONAL PARAMS")]
    pub(crate) opts: Option<String>,
}

#[derive(Subcommand)]
pub(crate) enum AssemblySubcommand {
    #[command(
        name = "auto",
        about = "Auto find clean reads and assembly them",
        long_about = None
    )]
    Auto(AutoArgs),
    #[command(name = "conf", about = "Runs SPAdes using a config file", long_about = None)]
    Conf(ConfArgs),
    #[command(name = "clean", about = "Cleans unused SPAdes files.")]
    Clean(CleanArgs),
}

#[derive(Args)]
pub(crate) struct AutoArgs {
    #[arg(
        short,
        long,
        default_value = "clean_reads",
        value_name = "CLEAN-READ DIR"
    )]
    pub(crate) dir: PathBuf,
    #[arg(
        short,
        long,
        default_value = "trimmed",
        value_name = "DIR NAME",
        help = "Specify QC files' directory name"
    )]
    pub(crate) specify: String,
    #[arg(short, long, value_name = "OUTPUT DIR")]
    pub(crate) output: Option<PathBuf>,
    #[arg(long, help = "Checks if the program can find the correct files")]
    pub(crate) dry_run: bool,
    #[arg(short, long, value_name = "THREAD-NUM")]
    pub(crate) threads: Option<usize>,
    #[arg(long, value_name = "OPTIONAL PARAMS")]
    pub(crate) opts: Option<String>,
    #[arg(
        long,
        help = "Keep all intermediate SPAdes files. Default is to keep only the contigs, scaffolds, and log files."
    )]
    pub(crate) keep_all: bool,
}

#[derive(Args)]
pub(crate) struct ConfArgs {
    #[arg(short, long, value_name = "INPUT")]
    pub(crate) input: PathBuf,
    #[arg(long, help = "Checks if the program detect the correct files")]
    pub(crate) dry_run: bool,
    #[arg(short, long, value_name = "THREAD-NUM")]
    pub(crate) threads: Option<usize>,
    #[arg(short, long, value_name = "OUTPUT DIR")]
    pub(crate) output: Option<PathBuf>,
    #[arg(long, value_name = "OPTIONAL PARAMS")]
    pub(crate) opts: Option<String>,
}

#[derive(Args)]
pub(crate) struct CleanArgs {
    #[arg(short, long, required = true, value_name = "DIR")]
    pub(crate) dir: PathBuf,
}
