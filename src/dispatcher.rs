use crate::data::*;
use crate::parser::*;
use std::thread::{self, JoinHandle};
use crate::genetic::*;

pub fn dispatch(file_name: String, cross_method : bool) -> JoinHandle<()> {
    println!("Spawning worker for {}...", file_name);
    let path = format!("{}{}", DATA_PATH, file_name);

    let matrix = get_mat(path);
    let name = file_name.clone();

    thread::spawn(move || {
        genetic_algorithm(name.clone(), matrix, if cross_method {cross_one} else {cross_two});
        println!("Thread for {} successfully completed its task!", name);
    })
}
