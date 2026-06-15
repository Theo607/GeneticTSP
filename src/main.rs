mod parser;
mod data;
mod tests;
mod dispatcher;
mod genetic;
mod math;
mod writer;

use data::FILE_NAMES;
use dispatcher::dispatch;

fn main() {
    println!("--- Starting Parallel Memetic TSP Solver ---");

    let mut worker_handles = Vec::new();

    for file_name in FILE_NAMES.iter() {
        let handle = dispatch(file_name.to_string(), true);
        worker_handles.push(handle);
    }
    // let handle = dispatch("dj38.tsp".to_string(), true);
    // worker_handles.push(handle);

    println!("\n All {} datasets are optimizing concurrently across your system cores!\n", worker_handles.len());

    for handle in worker_handles {
        handle.join().expect("A background worker panicked during execution!");
    }

    println!("\n--- All concurrent optimization tasks completed completely! ---");
}
