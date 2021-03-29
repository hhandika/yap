// mod qc;
mod assembly;
mod checker;
mod cli;
// mod utils;

use std::time::Instant;

fn main() {
    let time = Instant::now();
    checker::check_dependencies().unwrap();
    let duration = time.elapsed();

    println!("Execution time: {:?}", duration);
}
