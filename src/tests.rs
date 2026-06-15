use crate::data::*;
use crate::parser::*;

#[test]
fn test_parsing() {
    let file_path = format!("{}{}", DATA_PATH, "test.tsp"); 
    println!("Printing {}!", file_path);
    let mat = get_mat(file_path);

    print_mat(mat);
}

