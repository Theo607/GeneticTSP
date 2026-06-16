use rand::seq::SliceRandom;
use rand::{Rng, thread_rng};
use crate::math::delta;
use crate::writer::*;
use rayon::prelude::*;

#[derive(Clone)]
pub struct Permutation {
    pub perm :  Vec<u32>,
    pub len : u32
}

pub struct GenResult {
    pub castes : [Vec<Permutation> ; 4],
    pub gens : usize,
    pub best_path : Permutation,
}

#[derive(Clone)]
pub struct AlgorithmParams {
    pub population_divider : u32, 
    pub end_count : u32, 
    pub migration_frequency : u32,
    pub mutation_frequency : u32,
    pub turn_over_frequency : u32,
    pub search_frequency : u32,
    pub mimesis_frequency: u32,
    pub mimesis_iter : u32,
}

pub fn optimal_params() -> AlgorithmParams {
    // Params found through genetic analyzer
    AlgorithmParams { population_divider: 2, end_count: 80_000, migration_frequency: 2300, mutation_frequency: 250, turn_over_frequency: 80, search_frequency: 800, mimesis_frequency: 500, mimesis_iter: 3700 }
}

pub fn random_params() -> AlgorithmParams {
    let mut rng = thread_rng();
    
    let end_count = rng.gen_range(10_000..150_000);
    
    let population_divider = rng.gen_range(2..20);

    
    let mutation_frequency = rng.gen_range(10..500);
    let turn_over_frequency = rng.gen_range(5..100);

    let migration_frequency = rng.gen_range(100..2_500);
    let search_frequency = rng.gen_range(200..5_000);

    let mimesis_frequency = rng.gen_range(500..10_000);
    
    let max_mimesis_iter = (end_count / 10).max(100);
    let mimesis_iter = rng.gen_range(50..max_mimesis_iter);

    AlgorithmParams {
        population_divider,
        end_count,
        migration_frequency,
        mutation_frequency,
        turn_over_frequency,
        search_frequency,
        mimesis_frequency,
        mimesis_iter,
    }
}

pub fn default_params() -> AlgorithmParams {
    AlgorithmParams { population_divider: 4, end_count: 100_000, migration_frequency: 50, mutation_frequency: 250, turn_over_frequency: 5, search_frequency: 1000, mimesis_frequency: 100, mimesis_iter: 500 }
}

pub fn genetic_algorithm(
    params : AlgorithmParams,
    file_name : String,
    adj_mat : Vec<Vec<f32>>,
    cross: fn(&Permutation, &Permutation, &Vec<Vec<f32>>) -> Permutation
    ) -> GenResult
    {
    let n  = adj_mat[0].len();

    let target_population_size = n / params.population_divider as usize;

    let mut castes: [Vec<Permutation>; 4] = [
        populate(&adj_mat, target_population_size, n as u32),
        populate(&adj_mat, target_population_size, n as u32),
        populate(&adj_mat, target_population_size / 128, n as u32),
        populate(&adj_mat, target_population_size / 128, n as u32),
    ];
    
    let mut best_global = castes[0].first().expect("No permutation found").clone();

    let mut count = 0;
    let mut run = true;
    let mut i : u32 = 0;

    while run {
        // if i % 10_000  == 0{
        //     println!("Thread {} on gen {}", &file_name, i);
        // }
        count += 1;
        if count >= params.end_count {
            run = false;
            continue;
        }

        if i % params.migration_frequency == 0 {
            migrate(&mut castes);
        }

        if i % params.mutation_frequency == 0 {
            castes[0..2].par_iter_mut().for_each(|caste| {
                mutate_caste(&adj_mat, caste);
            });
        }

        if i % params.turn_over_frequency == 0 {
            castes.par_iter_mut().for_each(|caste| {
                cross_over(&adj_mat, caste, cross);
                purge(caste, target_population_size);
            });

        }

        if i % params.mimesis_frequency == 0 {
            castes[2..4].par_iter_mut().for_each(|caste| {
                mimesis_over(&adj_mat, caste, params.mimesis_iter);
            });
        }

        if i != 0 && i % (50 * params.mimesis_frequency) == 0 {
            castes[3..4].par_iter_mut().for_each(|caste| {
                mimesis_hard(&adj_mat, caste);
            });
        }

        if i % params.search_frequency == 0 {
            let mut found_better = false;

            for j in 0..4 {
                for p in castes[j].iter() {
                    if p.len < best_global.len {
                        best_global = p.clone(); 
                        found_better = true;

                    }
                }
            }

            if found_better {
                count = 0;
               println!("Thread {} on gen {} found a better solution: {}", &file_name, i, best_global.len);
                write_best(file_name.clone(), &best_global).expect("Couldn't save result.");
            }
        }

        i += 1;

    }
    let mut boosted = false;
    let mut len = best_global.len;
    let mut global_best : Vec<Permutation> = vec![best_global];
    for _ in 0..100 {
        mimesis_hard(&adj_mat, &mut global_best);
    }
    if global_best[0].len < len {
        boosted = true;
    }
    
    if boosted {
        println!("Boosted Thread {} on gen {} found a better solution: {}", &file_name, i, global_best[0].len);

        write_best(file_name.clone(), &global_best[0]).expect("Couldn't save result.");
    }

    

    GenResult { castes: castes, gens: i as usize, best_path: global_best[0].clone()}
}

fn mimesis_hard(adj_mat: &Vec<Vec<f32>>, caste: &mut Vec<Permutation>) {

    caste.iter_mut().for_each(|p| {

        let mut rng = thread_rng();
        let n = p.perm.len();

        if n < 2 { return; }
        for mut i in 0..n {

            for mut j in (i+1)..n {
                let d = delta(adj_mat, p, i, j);

                if d < 0.0 {
                    p.invert(i, j);
                    p.len = permutation_length(adj_mat, &p.perm);

                }
            }
        }
    });
}


fn mimesis_over(adj_mat: &Vec<Vec<f32>>, caste: &mut Vec<Permutation>, mimesis_iter : u32) {
    caste.par_iter_mut().for_each(|p| {
        mimesis(adj_mat, p, mimesis_iter);
    });
}

fn mimesis(adj_mat: &Vec<Vec<f32>>, perm: &mut Permutation, mimesis_iter : u32) {
    let mut rng = thread_rng();
    let n = perm.perm.len();
    if n < 2 { return; }

    let mut improved = true;
    let mut count = 0;
    while improved {
        count += 1;
        if count >= mimesis_iter {
            improved = false;
            continue;
        }
        let mut i = rng.gen_range(0..n);
        let mut j = rng.gen_range(0..n);

        if i == j { continue; }
        
        if i > j {
            std::mem::swap(&mut i, &mut j);
        }

        let d = delta(adj_mat, perm, i, j);
        
        if d < 0.0 {
            perm.invert(i, j);
            perm.len = permutation_length(adj_mat, &perm.perm);

            count = 0;
        }
    }
}
fn mutate_caste(adj_mat: &Vec<Vec<f32>>, caste: &mut Vec<Permutation>) {
    if caste.is_empty() { return; }

    let mut max_len = caste[0].len;
    let mut min_len = caste[0].len;
    for p in caste.iter() {
        if p.len > max_len { max_len = p.len; }
        if p.len < min_len { min_len = p.len; }
    }

    let range = (max_len - min_len) as f32;

    caste.par_iter_mut().for_each(|p| {
        let mut rng = thread_rng();
        let score = if range > 0.0 {
            1.0 - ((p.len - min_len) as f32 / range)
        } else {
            0.5 
        };

        if score > rng.gen_range(0.0..1.0) {
            mutate(adj_mat, p);
        }
    });
}

fn mutate(adj_mat: &Vec<Vec<f32>>, x : &mut Permutation) {
    let mut rng = thread_rng();

    for _ in 0..5 {
        let i = rng.gen_range(0..x.perm.len());
        let j = rng.gen_range(0..x.perm.len());

        x.invert(i, j);
    }

    let len = permutation_length(adj_mat, &x.perm);
    x.len = len;
}

fn cross_over(adj_mat: &Vec<Vec<f32>>, caste : &mut Vec<Permutation>, cross: fn(&Permutation, &Permutation, &Vec<Vec<f32>>) -> Permutation) {
    let count = (caste.len() as f64 * 0.2) as usize;
    let mut rng = thread_rng();

    for _ in 0..count {
        let i = rng.gen_range(0..caste.len());
        let j = rng.gen_range(0..caste.len());

        let child = cross(&caste[i], &caste[j], &adj_mat);
        caste.push(child);
    }
}

pub fn cross_one(x : &Permutation, y : &Permutation, adj_mat : &Vec<Vec<f32>>) -> Permutation {
    let mut rng = thread_rng();
    let count = (x.perm.len() as f64 * 0.15) as usize;
    let mut child_perm  = x.perm.clone();
    let mut to_fix : Vec<u32> = Vec::new();

    for _ in 0..count {
        let i = rng.gen_range(0..child_perm.len());

        let elem = child_perm.remove(i);
        to_fix.push(elem);
    }

    to_fix.sort_by_key(|&val| {
        y.perm.iter()
            .position(|&x| x == val)
            .unwrap_or(usize::MAX)
    });

    child_perm.append(&mut to_fix);

    let child_l = permutation_length(adj_mat, &child_perm);

    Permutation { perm: child_perm,  len: child_l}
}

pub fn cross_two(x: &Permutation, y: &Permutation, adj_mat: &Vec<Vec<f32>>) -> Permutation {
    let mut rng = thread_rng();
    let len = x.perm.len();
    if len == 0 { return Permutation { perm: vec![], len: 0 }; }

    let start = rng.gen_range(0..len);
    let end = rng.gen_range(start..len);

    let mut child_perm = vec![0; len]; 

    for i in start..=end {
        child_perm[i] = x.perm[i];
    }

    let mut y_idx = 0;
    for child_idx in 0..len {
        if child_idx >= start && child_idx <= end {
            continue;
        }

        while y_idx < len && child_perm.contains(&y.perm[y_idx]) {
            y_idx += 1;
        }

        if y_idx < len {
            child_perm[child_idx] = y.perm[y_idx];
            y_idx += 1;
        } else {
            if let Some(&fallback) = x.perm.iter().find(|e| !child_perm.contains(e)) {
                child_perm[child_idx] = fallback;
            }
        }
    }

    let child_l = permutation_length(adj_mat, &child_perm);
    Permutation { perm: child_perm, len: child_l }
}

fn purge(caste: &mut Vec<Permutation>, target_size: usize) {
    caste.sort_by_key(|p| p.len);
    if caste.len() > target_size {
        caste.truncate(target_size);
    }
}

fn migrate(castes : &mut [Vec<Permutation>; 4]) {
    castes[0].sort_by_key(|p| p.len);
    let count = (castes[0].len() as f32 * 0.1) as usize;
    if count > 0 {
        let mut promoted : Vec<Permutation> = castes[0].drain(0..count).collect();
        castes[1].append(&mut promoted);
    }

    for c in 1..=2 {
        let count = (castes[c].len() as f32 * 0.05) as usize;
        if count > 0 {
            let mut promoted : Vec<Permutation> = castes[c].drain(0..count).collect();
            castes[c+1].append(&mut promoted);

        }

        let start = castes[c].len() - count;
        if start < castes[c].len() {
            let mut demoted : Vec<Permutation> = castes[c].drain(start..).collect();
            castes[c-1].append(&mut demoted);
        }
    }
    castes[3].sort_by_key(|p| p.len);
    let count = (castes[3].len() as f32 * 0.1) as usize;
    let start = castes[3].len() - count;
    if start < castes[3].len() {
        let mut demoted : Vec<Permutation> = castes[3].drain(start..).collect();
        castes[2].append(&mut demoted);
    }
}

fn populate(adj_mat : &Vec<Vec<f32>>, population_size : usize, perm_size : u32) -> Vec<Permutation> {
    let mut population : Vec<Permutation> = Vec::new();
    for _ in 0..population_size {
        let mut perm : Vec<u32> = (1..=perm_size).collect();
        let mut rng = thread_rng();
        perm.shuffle(&mut rng);

        let mut len = permutation_length(&adj_mat, &perm);


        population.push(Permutation {perm, len});
    }

    population
}

fn permutation_length(adj_mat: &Vec<Vec<f32>>, permutation: &Vec<u32>) -> u32 {
    let mut acc: u32 = 0;
    let n = permutation.len();
    if n < 2 { return 0; }

    for i in 0..n-1 {
        let from = permutation[i] as usize;
        let to = permutation[i+1] as usize;
        acc += adj_mat[from - 1][to - 1].round() as u32;
    }

    let start = permutation[0] as usize;
    let end = permutation[n-1] as usize;
    
    acc += adj_mat[end - 1][start - 1].round() as u32;

    acc
}
