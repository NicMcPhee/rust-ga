pub mod args;

use std::ops::Not;

use anyhow::{ensure, Result};
use clap::Parser;
use ec_core::{
    distributions::collection::ConvertToCollectionGenerator,
    generation::Generation,
    individual::{ec::WithScorer, scorer::FnScorer},
    operator::{
        genome_extractor::GenomeExtractor,
        genome_scorer::GenomeScorer,
        mutator::Mutate,
        selector::{
            best::Best, lexicase::Lexicase, tournament::Tournament, weighted::Weighted, Select,
            Selector,
        },
        Composable,
    },
    test_results::{self, TestResults},
};
use ec_linear::mutator::umad::Umad;
use num_traits::Float;
use ordered_float::OrderedFloat;
use push::{
    evaluation::cases::{Case, Cases, WithTarget},
    genome::plushy::{GeneGenerator, Plushy},
    instruction::{variable_name::VariableName, FloatInstruction},
    push_vm::{program::PushProgram, push_state::PushState, HasStack, State},
    vec_into,
};
use rand::{distributions::Distribution, thread_rng};

use crate::args::{Args, RunModel};

/*
* This is an implementation of the "simple regression" problem from the

* Clojush implementation of PushGP:
* https://github.com/lspector/Clojush/blob/e2c9d8c830715f7d1e644f6205c192b9e5ceead2/src/clojush/problems/demos/simple_regression.clj
*/

const PENALTY_VALUE: f64 = 1_000.0;

type Of64 = OrderedFloat<f64>;

fn target_fn(input: Of64) -> Of64 {
    input.powi(3) - Of64::from(2) * input.powi(2) - input
}

fn build_push_state(
    program: impl DoubleEndedIterator<Item = PushProgram> + ExactSizeIterator,
    input: Of64,
) -> PushState {
    #[allow(clippy::unwrap_used)]
    PushState::builder()
        .with_max_stack_size(1000)
        .with_program(program)
        // This will return an error if the program is longer than the allowed
        // max stack size.
        // We arguably should check that and return an error here.
        .unwrap()
        .with_float_input("x", input)
        .build()
}

fn score_program(
    program: impl DoubleEndedIterator<Item = PushProgram> + ExactSizeIterator,
    Case { input, output }: Case<Of64>,
) -> Of64 {
    let state = build_push_state(program, input);
    #[allow(clippy::option_if_let_else)]
    match state.run_to_completion() {
        Ok(final_state) => final_state
            .stack::<Of64>()
            .top()
            .map_or(Of64::from(PENALTY_VALUE), |answer| (answer - output).abs()),

        Err(_) => {
            // Do some logging, perhaps?
            Of64::from(PENALTY_VALUE)
        }
    }
}

fn score_genome(
    genome: &Plushy,
    training_cases: &Cases<Of64>,
) -> TestResults<test_results::Error<Of64>> {
    let program: Vec<PushProgram> = genome.clone().into();

    training_cases
        .iter()
        .map(|&case| score_program(program.iter().cloned(), case))
        .collect()
}

fn main() -> Result<()> {
    let Args {
        run_model,
        population_size,
        max_initial_instructions,
        // FIXME: Actually use this
        max_genome_length: _,
        num_generations,
    } = Args::parse();

    let mut rng = thread_rng();

    // Inputs from -4 (inclusive) to 4 (exclusive) in increments of 0.25.
    let training_cases = (-4 * 4..4 * 4)
        .map(|n| Of64::from(n) / 4.0)
        .with_target(|&i| target_fn(i));

    /*
     * The `scorer` will need to take an evolved program (sequence of
     * instructions) and run it on all the inputs from -4 (inclusive) to 4
     * (exclusive) in increments of 0.25, collecting together the errors,
     * i.e., the absolute difference between the returned value and the
     * expected value.
     *
     * The target polynomial is x^3 - 2x^2 - x
     */
    let scorer = FnScorer(|genome: &Plushy| score_genome(genome, &training_cases));

    let num_test_cases = 10;

    let selector = Weighted::new(Best, 1)
        .with_selector(Lexicase::new(num_test_cases), 5)
        .with_selector(Tournament::new(2), population_size - 1);

    let instruction_set = vec_into![
        FloatInstruction::Add,
        FloatInstruction::Subtract,
        FloatInstruction::Multiply,
        FloatInstruction::ProtectedDivide,
        VariableName::from("x")
    ];

    let gene_generator = GeneGenerator::with_uniform_close_probability(&instruction_set)?;

    let population = gene_generator
        .to_collection_generator(max_initial_instructions)
        .with_scorer(scorer)
        .into_collection_generator(population_size)
        .sample(&mut rng);

    ensure!(population.is_empty().not());

    let best = Best.select(&population, &mut rng)?;
    println!("Best initial individual is {best:?}");

    let umad = Umad::new(0.1, 0.1, &gene_generator);

    let make_new_individual = Select::new(selector)
        .then(GenomeExtractor)
        .then(Mutate::new(umad))
        .wrap::<GenomeScorer<_, _>>(scorer);

    let mut generation = Generation::new(make_new_individual, population);

    // TODO: It might be useful to insert some kind of logging system so we can
    // make this less imperative in nature.

    for generation_number in 0..num_generations {
        match run_model {
            RunModel::Serial => generation.serial_next()?,
            RunModel::Parallel => generation.par_next()?,
        }

        let best = Best.select(generation.population(), &mut rng)?;
        // TODO: Change 2 to be the smallest number of digits needed for
        // num_generations-1.
        println!("Generation {generation_number:2} best is {best:#?}");

        if best.test_results.total_result.error == OrderedFloat(0.0) {
            println!("SUCCESS");
            break;
        }
    }

    Ok(())
}
