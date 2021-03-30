// mod qc;
mod assembly;
mod checker;
mod cli;
mod utils;

use clap::crate_version;
use std::time::Instant;

fn main() {
    let version = crate_version!();
    let time = Instant::now();
    cli::get_cli(version);
    let duration = time.elapsed();

    if duration.as_secs() < 60 {
        println!("Execution time: {:?}", duration);
    } else {
        utils::print_formatted_duration(duration.as_secs());
    }
}
