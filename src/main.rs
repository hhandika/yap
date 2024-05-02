mod assembly;
mod checker;
mod cli;
mod init;
mod qc;
mod utils;

#[macro_use]
extern crate lazy_static;

use std::time::Instant;

fn main() {
    let time = Instant::now();
    cli::cli::parse_cli();
    let duration = time.elapsed();

    if duration.as_secs() < 60 {
        println!("Execution time: {:?}", duration);
    } else {
        utils::print_formatted_duration(duration.as_secs());
    }
}
