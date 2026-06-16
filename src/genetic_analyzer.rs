use crate::genetic::{optimal_params, AlgorithmParams, GenResult, cross_two, genetic_algorithm, random_params};
use crate::parser::get_mat;
use crate::data::*;

pub fn measure(model_params : AlgorithmParams, test_subject : String) -> f64 {
    let path = format!("{}{}", DATA_PATH, test_subject.clone());
    let matrix = get_mat(path);
    let results : GenResult = genetic_algorithm(model_params, test_subject, matrix, cross_two); 

    let generations = results.gens as f64;
    let best_path_length = results.best_path.len as f64;
    
    let total_n: usize = results.castes.iter().map(|c| c.len()).sum();
    
    if total_n <= 1 {
        return f64::MAX; 
    }

    let mut total_sum = 0.0;
    for c in results.castes.iter() {
        for p in c {
            total_sum += p.len as f64;
        }
    }
    let global_mean = total_sum / total_n as f64;

    let mut variance = 0.0;
    for c in results.castes.iter() {
        for p in c {
            variance += (global_mean - p.len as f64).powi(2);
        }
    }
    variance /= (total_n - 1) as f64;
    
    let epsilon = 1e-6; 
    let std_dev = variance.sqrt() + epsilon;

    
    let path_score = best_path_length;
    let time_penalty = 0.5 * generations.ln();
    
    let diversity_penalty = 0.1 * std_dev.ln().abs(); 

    path_score + time_penalty + diversity_penalty
}

pub struct OptimizationResult {
    pub best_params: AlgorithmParams,
    pub best_score: f64,
    pub params_avg: f64,
}

pub fn optimize_parameters(
    test_subject: String, 
    num_iterations: usize, 
    eval_runs_per_param: usize
) -> OptimizationResult {
    
    let mut best_params: Option<AlgorithmParams> = None;
    let mut best_score = f64::MAX;
    let mut params_avg = 0.0;

    println!("Starting parameter optimization ({} candidates)...", num_iterations);

    for i in 0..num_iterations {
        println!("Candidate {}", i);
        let candidate_params = random_params();
        
        let mut total_score = 0.0;
        for i in 0..eval_runs_per_param {
            total_score += measure(candidate_params.clone(), test_subject.clone());
        }
        let average_score = total_score / eval_runs_per_param as f64;

        params_avg += average_score;

        if average_score < best_score {
            best_score = average_score;
            best_params = Some(candidate_params);
        }
    }

    OptimizationResult {
        best_params: best_params.expect("Optimization iterations must be > 0"),
        best_score: best_score,
        params_avg: params_avg / num_iterations as f64,
    }
}


pub fn optimize() {
    for file_name in FILE_NAMES[2..3].iter() {

        let name = file_name.to_string();
        let result = optimize_parameters(name.clone(), 10, 10);
        
        println!("\n==================================================");
        println!("Results of optimizing target: {}", name);
        println!("==================================================");
        println!("Best Evaluation Score: {:.4}", result.best_score);
        println!("Avg Model Score: {:.4}", result.params_avg);
        println!("--------------------------------------------------");
        println!("Optimized Hyperparameters:");
        println!("  • population_divider   : {}", result.best_params.population_divider);
        println!("  • end_count            : {}", result.best_params.end_count);
        println!("  • migration_frequency  : {}", result.best_params.migration_frequency);
        println!("  • mutation_frequency   : {}", result.best_params.mutation_frequency);
        println!("  • turn_over_frequency  : {}", result.best_params.turn_over_frequency);
        println!("  • search_frequency     : {}", result.best_params.search_frequency);
        println!("  • mimesis_frequency    : {}", result.best_params.mimesis_frequency);
        println!("  • mimesis_iter         : {}", result.best_params.mimesis_iter);
        println!("==================================================\n");
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TuningAxis {
    PopulationDivider,
    EndCount,
    MigrationFrequency,
    MutationFrequency,
    TurnOverFrequency,
    SearchFrequency,
    MimesisFrequency,
    MimesisIter,
}

pub const ALL_AXES: [TuningAxis; 8] = [
    TuningAxis::PopulationDivider,
    TuningAxis::EndCount,
    TuningAxis::MigrationFrequency,
    TuningAxis::MutationFrequency,
    TuningAxis::TurnOverFrequency,
    TuningAxis::SearchFrequency,
    TuningAxis::MimesisFrequency,
    TuningAxis::MimesisIter,
];

fn modify_param_on_axis(
    base: &AlgorithmParams, 
    axis: TuningAxis, 
    value: u32
) -> AlgorithmParams {
    let mut modified = base.clone();
    match axis {
        TuningAxis::PopulationDivider  => modified.population_divider = value,
        TuningAxis::EndCount           => modified.end_count = value,
        TuningAxis::MigrationFrequency => modified.migration_frequency = value,
        TuningAxis::MutationFrequency  => modified.mutation_frequency = value,
        TuningAxis::TurnOverFrequency  => modified.turn_over_frequency = value,
        TuningAxis::SearchFrequency    => modified.search_frequency = value,
        TuningAxis::MimesisFrequency   => modified.mimesis_frequency = value,
        TuningAxis::MimesisIter        => modified.mimesis_iter = value,
    }
    modified
}

pub fn tune_single_axis(
    axis: TuningAxis,
    base_params: &AlgorithmParams,
    test_subject: String,
    candidate_values: Vec<u32>,
    eval_runs: usize,
) -> (u32, f64) {
    let mut best_val = match axis {
        TuningAxis::PopulationDivider  => base_params.population_divider,
        TuningAxis::EndCount           => base_params.end_count,
        TuningAxis::MigrationFrequency => base_params.migration_frequency,
        TuningAxis::MutationFrequency  => base_params.mutation_frequency,
        TuningAxis::TurnOverFrequency  => base_params.turn_over_frequency,
        TuningAxis::SearchFrequency    => base_params.search_frequency,
        TuningAxis::MimesisFrequency   => base_params.mimesis_frequency,
        TuningAxis::MimesisIter        => base_params.mimesis_iter,
    };
    
    let mut best_score = f64::MAX;

    println!("--- Tuning Axis: {:?} ---", axis);

    for &val in &candidate_values {
        if (matches!(axis, TuningAxis::PopulationDivider) && val == 0) {
            continue;
        }

        let candidate_params = modify_param_on_axis(base_params, axis, val);

        let mut total_score = 0.0;
        for _ in 0..eval_runs {
            total_score += measure(candidate_params.clone(), test_subject.clone());
        }
        let avg_score = total_score / eval_runs as f64;

        println!("  Testing value: {} -> Avg Score: {:.4}", val, avg_score);

        if avg_score < best_score {
            best_score = avg_score;
            best_val = val;
        }
    }

    println!(">> Best value for {:?}: {} (Score: {:.4})\n", axis, best_val, best_score);
    (best_val, best_score)
}

pub fn optimize_one_by_one() {
    let mut current_best_params = optimal_params();
    let test_subject = FILE_NAMES[2].to_string();
    
    let eval_runs = 5; 

    println!("Starting Coordinate-Wise Hyperparameter Tuning for {}", test_subject);
    
    let (best_div, _) = tune_single_axis(
        TuningAxis::PopulationDivider,
        &current_best_params,
        test_subject.clone(),
        vec![2, 4, 6, 8, 12, 16],
        eval_runs,
    );
    current_best_params.population_divider = best_div;

    let (best_end, _) = tune_single_axis(
        TuningAxis::EndCount,
        &current_best_params,
        test_subject.clone(),
        vec![30_000, 45_000, 60_000, 80_000, 100_000],
        eval_runs,
    );
    current_best_params.end_count = best_end;

    let (best_mut, _) = tune_single_axis(
        TuningAxis::MutationFrequency,
        &current_best_params,
        test_subject.clone(),
        vec![25, 50, 90, 150, 250],
        eval_runs,
    );
    current_best_params.mutation_frequency = best_mut;

    let (best_search, _) = tune_single_axis(
        TuningAxis::SearchFrequency,
        &current_best_params,
        test_subject.clone(),
        vec![100, 300, 500, 800, 1200],
        eval_runs,
    );
    current_best_params.search_frequency = best_search;

    let (best_mim_freq, _) = tune_single_axis(
        TuningAxis::MimesisFrequency,
        &current_best_params,
        test_subject.clone(),
        vec![500, 1000, 2000, 4000],
        eval_runs,
    );
    current_best_params.mimesis_frequency = best_mim_freq;

    println!("==================================================");
    println!("Final Optimized Parameters After Coordinate Sweep:");
    println!("================================================--");
    println!("  • population_divider   : {}", current_best_params.population_divider);
    println!("  • end_count            : {}", current_best_params.end_count);
    println!("  • migration_frequency  : {}", current_best_params.migration_frequency);
    println!("  • mutation_frequency   : {}", current_best_params.mutation_frequency);
    println!("  • turn_over_frequency  : {}", current_best_params.turn_over_frequency);
    println!("  • search_frequency     : {}", current_best_params.search_frequency);
    println!("  • mimesis_frequency    : {}", current_best_params.mimesis_frequency);
    println!("  • mimesis_iter         : {}", current_best_params.mimesis_iter);
    println!("==================================================");
}
