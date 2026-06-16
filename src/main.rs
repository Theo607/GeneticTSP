mod parser;
mod data;
mod tests;
mod dispatcher;
mod genetic;
mod math;
mod writer;
mod genetic_analyzer;

use data::FILE_NAMES;
use dispatcher::dispatch;
use genetic_analyzer::optimize_one_by_one;

fn main() {
    println!("--- Starting Parallel Memetic TSP Solver ---");

    let mut worker_handles = Vec::new();

    let mut start = 0;
    let mut end = 10;


    for file_name in FILE_NAMES[start..end].iter() {
        let handle = dispatch(file_name.to_string(), false);
        worker_handles.push(handle);
    }

    println!("\n All {} datasets are optimizing concurrently across your system cores!\n", worker_handles.len());

    for handle in worker_handles {
        handle.join().expect("A background worker panicked during execution!");
    }

    println!("\n--- All concurrent optimization tasks completed completely! ---");
}
