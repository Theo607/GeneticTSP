use std::fs::File;
use std::io::{BufWriter, Write, Result};
use crate::genetic::Permutation;

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
