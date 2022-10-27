use clap::Parser;

#[derive(clap::ValueEnum, Copy, Clone, Debug)]
pub enum RunModel {
    Serial,
    Parallel,
}

#[derive(clap::ValueEnum, Copy, Clone, Debug)]
pub enum TargetProblem {
    CountOnes,
    Hiff,
}

/// Simple genetic algorithm in Rust
#[derive(Parser, Debug, Copy, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Should we use parallelism when doing the run?
    #[clap(short, long, value_enum, default_value_t = RunModel::Parallel)]
    pub run_model: RunModel,

    /// The target problem to run
    #[clap(short, long, value_enum, default_value_t = TargetProblem::Hiff)]
    pub target_problem: TargetProblem,

    /// Population size
    #[clap(short, long, value_parser, default_value_t = 100)]
    pub population_size: usize,

    /// Number of bits in bit strings
    #[clap(short, long, value_parser, default_value_t = 128)]
    pub bit_length: usize,

    /// Number of generations to run
    #[clap(short, long, value_parser, default_value_t = 100)]
    pub num_generations: usize,
}
