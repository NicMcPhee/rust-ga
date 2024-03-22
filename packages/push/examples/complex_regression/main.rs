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
        selector::{best::Best, lexicase::Lexicase, Select, Selector},
        Composable,
    },
    test_results::{self, TestResults},
    uniform_distribution_of,
};
use ec_linear::mutator::umad::Umad;
use ordered_float::OrderedFloat;
use push::{
    evaluation::cases::{Case, Cases},
    genome::plushy::{ConvertToGeneGenerator, Plushy},
    instruction::{variable_name::VariableName, FloatInstruction, PushInstruction},
    push_vm::{program::PushProgram, push_state::PushState, HasStack, State},
};
use rand::{prelude::Distribution, thread_rng};

use crate::args::{Args, RunModel};

/*
 * This is an implementation of the "complex regression" problem from the
 * Propeller implementation of PushGP:
 * https://github.com/lspector/propeller/blob/71d378f49fdf88c14dda88387291c9c7be0f1277/src/propeller/problems/complex_regression.cljc
 */

// The penalty value to use when an evolved program doesn't have an expected
// "return" value on the appropriate stack at the end of its execution.
const PENALTY_VALUE: f64 = 1_000.0;

type Of64 = OrderedFloat<f64>;

/// The target polynomial is (x^3 + 1)^3 + 1
/// i.e., x^9 + 3x^6 + 3x^3 + 2
fn target_fn(input: Of64) -> Of64 {
    let sub_expr = input * input * input + 1.0;
    sub_expr * sub_expr * sub_expr + 1.0
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
        Ok(final_state) => OrderedFloat(
            final_state
                .stack::<Of64>()
                .top()
                .map_or(PENALTY_VALUE, |answer| (answer - output).abs()),
        ),
        Err(_) => {
            // Do some logging, perhaps?
            OrderedFloat(PENALTY_VALUE)
        }
    }
}

fn score_genome(
    genome: &Plushy,
    training_cases: &Cases<Of64>,
) -> TestResults<test_results::Error<Of64>> {
    let program = Vec::<PushProgram>::from(genome.clone());
    let errors: TestResults<test_results::Error<Of64>> = training_cases
        .iter()
        .map(|&case| score_program(program.iter().cloned(), case))
        .collect();
    errors
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Inputs from -4 (inclusive) to 4 (exclusive) in increments of 0.25.
    let training_inputs = (-4 * 4..4 * 4).map(|n| Of64::from(n) / 4.0);
    let training_cases = Cases::from_inputs(training_inputs, |&i| target_fn(i));

    // The range want is -4 1/8, -3 7/8, -3 5/8, ..., 3 7/8, 4 1/8.
    // I have to multiply that by 8 to get integer values, so:
    // -33, -31, -29, ..., 31, 33.
    let testing_inputs = (-33..=33).step_by(2).map(|n| Of64::from(n) / 8.0);
    let _testing_cases = Cases::from_inputs(testing_inputs, |&i| target_fn(i));

    /*
     * The `scorer` will need to take an evolved program (sequence of
     * instructions) and run it on all the inputs from -4 (inclusive) to 4
     * (exclusive) in increments of 0.25, collecting together the errors,
     * i.e., the absolute difference between the returned value and the
     * expected value.
     */
    let scorer = FnScorer(
        |genome: &Plushy| -> TestResults<test_results::Error<Of64>> {
            score_genome(genome, &training_cases)
        },
    );

    let selector = Lexicase::new(training_cases.len());

    let mut rng = thread_rng();

    let gene_generator = uniform_distribution_of![<PushInstruction>
        FloatInstruction::Add,
        FloatInstruction::Subtract,
        FloatInstruction::Multiply,
        FloatInstruction::ProtectedDivide,
        FloatInstruction::Dup,
        FloatInstruction::Push(OrderedFloat(0.0)),
        FloatInstruction::Push(OrderedFloat(1.0)),
        VariableName::from("x")
    ]
    .into_gene_generator();

    let population = gene_generator
        .to_collection_generator(args.max_initial_instructions)
        .with_scorer(scorer)
        .into_collection_generator(args.population_size)
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

    for generation_number in 0..args.num_generations {
        match args.run_model {
            RunModel::Serial => generation.serial_next()?,
            RunModel::Parallel => generation.par_next()?,
        }

        let best = Best.select(generation.population(), &mut rng)?;
        // TODO: Change 2 to be the smallest number of digits needed for
        // args.num_generations-1.
        println!("Generation {generation_number:2} best is {best:#?}");

        if best.test_results.total_result.error == OrderedFloat(0.0) {
            println!("SUCCESS");
            break;
        }
    }

    Ok(())
}
