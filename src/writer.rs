use std::io::{BufWriter, Write, Result};
use crate::genetic::Permutation;
use std::fs::{self, File};
use crate::data::City;

pub const DATA_OUT: &str = "output/";

pub fn write_best(file_name: String, perm: &Permutation) -> Result<()> {
    let file_path = format!("{}{}", DATA_OUT, file_name);
    
    let file = File::create(file_path)?;
    let mut writer = BufWriter::new(file);

    writeln!(writer, "Distance: {}", perm.len)?;

    for (idx, node) in perm.perm.iter().enumerate() {
        if idx == perm.perm.len() - 1 {
            write!(writer, "{}", node)?; 
        } else {
            write!(writer, "{} ", node)?;
        }
    }
    writeln!(writer)?; 

    writer.flush()?;

    Ok(())
}



/// Generates an SVG file representing the TSP route.
/// 
/// * `file_path` - The destination path for the .svg file (e.g., "output/route.svg").
/// * `cities` - The original vector of cities parsed from the .tsp file.
/// * `tour` - A slice of 1-based indices representing the sequence of visited cities 
///            (e.g., [16, 14, 13, ...]).

/// Generates an SVG file representing the TSP route.
pub fn export_tour_to_svg(
    file_path: &str, 
    cities: &[City], 
    tour: &[usize]
) -> std::io::Result<()> {
    if cities.is_empty() || tour.is_empty() { 
        return Ok(()); 
    }

    // Ensure output directories exist automatically
    if let Some(parent) = std::path::Path::new(file_path).parent() {
        fs::create_dir_all(parent)?;
    }

    let width = 800.0;
    let height = 600.0;
    let padding = 40.0; // Slightly tighter padding since text labels are gone

    // Determine boundaries for scaling
    let mut min_x = cities[0].x; 
    let mut max_x = cities[0].x;
    let mut min_y = cities[0].y; 
    let mut max_y = cities[0].y;
    
    for city in cities {
        if city.x < min_x { min_x = city.x; }
        if city.x > max_x { max_x = city.x; }
        if city.y < min_y { min_y = city.y; }
        if city.y > max_y { max_y = city.y; }
    }

    let dx = if max_x == min_x { 1.0 } else { max_x - min_x };
    let dy = if max_y == min_y { 1.0 } else { max_y - min_y };
    let scale = f32::min((width - 2.0 * padding) / dx, (height - 2.0 * padding) / dy);

    let transform = |c: &City| -> (f32, f32) {
        (
            padding + (c.x - min_x) * scale, 
            height - (padding + (c.y - min_y) * scale)
        )
    };

    let mut svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="100%" height="100%" style="background-color: #111827;">
"##, 
        width, height
    );

    // --- 1. Draw Paths/Edges ---
    svg.push_str("  <path d=\"");
    for (i, &city_idx) in tour.iter().enumerate() {
        let idx = city_idx - 1;
        if idx >= cities.len() { continue; }
        
        let (sx, sy) = transform(&cities[idx]);
        if i == 0 {
            svg.push_str(&format!("M {},{} ", sx, sy));
        } else {
            svg.push_str(&format!("L {},{} ", sx, sy));
        }
    }
    
    // Close path loop back to beginning
    if let Some(&first_idx) = tour.first() {
        let (sx, sy) = transform(&cities[first_idx - 1]);
        svg.push_str(&format!("L {},{}", sx, sy));
    }
    svg.push_str("\" fill=\"none\" stroke=\"#10b981\" stroke-width=\"1.8\" stroke-linejoin=\"round\" stroke-linecap=\"round\" />\n");

    // --- 2. Draw Mini Nodes/Cities ---
    for &city_idx in tour {
        let idx = city_idx - 1;
        if idx >= cities.len() { continue; }
        let (sx, sy) = transform(&cities[idx]);

        // Tiny elegant point marker with no overlapping text labels
        svg.push_str(&format!(
            "  <circle cx=\"{:.2}\" cy=\"{:.2}\" r=\"1.5\" fill=\"#ffffff\" opacity=\"0.9\" />\n",
            sx, sy
        ));
    }

    svg.push_str("</svg>");

    let mut file = File::create(file_path)?;
    file.write_all(svg.as_bytes())?;

    Ok(())
}
