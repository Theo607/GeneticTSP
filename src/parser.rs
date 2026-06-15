use std::fs::File;
use std::io::{BufRead, BufReader};
use crate::data::City;

pub fn parse_file(path : String) -> std::io::Result<Vec<City>> {
    let file  = File::open(path)?;

    let mut input = String::new();

    let mut reader = BufReader::new(file);
    let mut flag = true;

    let mut cities : Vec<City> = Vec::new();

    while reader.read_line(&mut input)? > 0 {
        let line = input.trim();

        if line == "EOF" { break; }

        if line == "NODE_COORD_SECTION" {
            flag = false;
            input.clear();
            continue;
        }

        if flag {
            input.clear();
            continue;
        }

        let mut words = line.split_whitespace();

        words.next();

        if let (Some(x_str), Some(y_str)) = (words.next(), words.next()) {
            if let (Ok(x), Ok(y)) = (x_str.parse::<f32>(), y_str.parse::<f32>()) {
                cities.push(City {x, y});
            }
        }

        input.clear();

    }

    Ok(cities)

}

pub fn adjecency_matrix(cities : Vec<City>) -> Vec<Vec<f32>> {
    let n = cities.len();
    let mut adj_mat  = vec![vec![0.0 as f32 ; n] ; n];
    for i in 0..n {
        for j in 0..n {
            let dx = cities[i].x - cities[j].x;
            let dy = cities[i].y - cities[j].y;
            let dist = (dx*dx + dy*dy).sqrt();

            adj_mat[i][j] = dist;
        }
    }
    adj_mat
}

pub fn get_mat(path : String) -> Vec<Vec<f32>> {
    let cities = parse_file(path.clone()).unwrap_or_else(|err| {
        panic!("Failed to parse file at '{}'! Error: {}", path, err);
    });

    adjecency_matrix(cities)
}

pub fn print_mat(mat : Vec<Vec<f32>>) -> () {
    for row in mat {
        for val in row {
            print!("{:<8.2} ", val);
        }
        println!();
    }
}
