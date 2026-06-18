mod parser;
mod data;
mod tests;
mod dispatcher;
mod genetic;
mod math;
mod writer;
mod genetic_analyzer;

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use data::FILE_NAMES;
use parser::parse_file;
use writer::export_tour_to_svg;

/// Helper function to parse a tour sequence from a generated result text file.
/// It looks for the array line or list of integers following the "Distance:" string.
fn parse_tour_file(path: &str) -> std::io::Result<Vec<usize>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut tour = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        
        // Skip distance header lines or empty lines
        if trimmed.is_empty() || trimmed.starts_with("Distance:") {
            continue;
        }

        // Parse whitespace-separated numbers into a sequence vector
        for word in trimmed.split_whitespace() {
            if let Ok(node_idx) = word.parse::<usize>() {
                tour.push(node_idx);
            }
        }
    }
    Ok(tour)
}

fn main() {
    println!("--- Starting TSP SVG Batch Renderer ---");

    // Define the directories matching your tree layout
    let assets_dir = "assets";
    let sub_folders = vec!["output/cross1", "output/cross2"];

    let mut generated_count = 0;

    // Iterate through all file names defined in your data crate
    for file_name in FILE_NAMES.iter() {
        let base_name = file_name.replace(".tsp", "");
        let asset_path = format!("{}/{}", assets_dir, file_name);

        // 1. Verify the base asset coordinates file exists and parse it
        if !Path::new(&asset_path).exists() {
            println!("Skipping {}: Asset source file not found.", file_name);
            continue;
        }
        
        let cities = match parse_file(asset_path.clone()) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to parse asset data '{}': {}", asset_path, e);
                continue;
            }
        };

        // 2. Search through both cross1 and cross2 output folders for matching tour results
        for folder in &sub_folders {
            let tour_path = format!("{}/{}", folder, file_name);

            if Path::new(&tour_path).exists() {
                // 3. Parse the calculated tour order out of your file
                match parse_tour_file(&tour_path) {
                    Ok(tour) => {
                        // Place the generated SVG right alongside the solution file
                        let svg_output_path = format!("{}/{}.svg", folder, base_name);

                        // 4. Draw and save the vector graphic
                        if let Err(e) = export_tour_to_svg(&svg_output_path, &cities, &tour) {
                            eprintln!("Error writing SVG for {}: {}", tour_path, e);
                        } else {
                            println!(" Rendered SVG -> {}", svg_output_path);
                            generated_count += 1;
                        }
                    }
                    Err(e) => eprintln!("Failed to read tour indices at '{}': {}", tour_path, e),
                }
            }
        }
    }

    println!("\n--- Done! Successfully generated {} SVG graphics across your output tree. ---", generated_count);
}
