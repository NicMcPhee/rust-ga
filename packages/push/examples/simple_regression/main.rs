pub mod args;

use std::ops::Not;

use anyhow::{ensure, Result};
use clap::Parser;
use ec_core::{
    distributions::collection::ConvertToCollectionGenerator,
    generation::Generation,
    individual::{
        ec::{EcIndividual, WithScorer},
        scorer::FnScorer,
    },
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
    uniform_distribution_of,
};
use ec_linear::mutator::umad::Umad;
use ordered_float::OrderedFloat;
use push::{
    genome::plushy::{ConvertToGeneGenerator, Plushy},
    instruction::{variable_name::VariableName, FloatInstruction, PushInstruction},
    push_vm::{program::PushProgram, push_state::PushState, HasStack, State},
};
use rand::{distributions::Distribution, thread_rng};

use crate::args::{Args, RunModel};

/*
 * This is an implementation of the "simple regression" problem from the
 * Clojush implementation of PushGP:
 * https://github.com/lspector/Clojush/blob/e2c9d8c830715f7d1e644f6205c192b9e5ceead2/src/clojush/problems/demos/simple_regression.clj
 */

fn main() -> Result<()> {
    // Using `Error` in `TestResults<Error>` will have the run favor smaller
    // values, where using `Score` (e.g., `TestResults<Score>`) will have the run
    // favor larger values.
    type Pop = Vec<EcIndividual<Plushy, TestResults<test_results::Error<OrderedFloat<f64>>>>>;
    // The penalty value to use when an evolved program doesn't have an expected
    // "return" value on the appropriate stack at the end of its execution.
    let penalty_value: OrderedFloat<f64> = OrderedFloat::from(1_000.0);

    let args = Args::parse();

    // Inputs from -4 (inclusive) to 4 (exclusive) in increments of 0.25.
    let training_inputs = (-4 * 4..4 * 4)
        .map(|n| OrderedFloat(f64::from(n) / 4.0))
        .collect::<Vec<_>>();

    /*
     * The `scorer` will need to take an evolved program (sequence of
     * instructions) and run it on all the inputs from -4 (inclusive) to 4
     * (exclusive) in increments of 0.25, collecting together the errors,
     * i.e., the absolute difference between the returned value and the
     * expected value.
     *
     * The target polynomial is x^3 - 2x^2 - x
     */
    let scorer = FnScorer(
        |genome: &Plushy| -> TestResults<test_results::Error<OrderedFloat<f64>>> {
            let program = Vec::<PushProgram>::from(genome.clone());
            let errors: TestResults<test_results::Error<OrderedFloat<f64>>> = training_inputs
                .iter()
                .map(|&input| {
                    #[allow(clippy::unwrap_used)]
                    let state = PushState::builder()
                        .with_max_stack_size(1000)
                        .with_program(program.clone())
                        // This will return an error if the program is longer than the allowed
                        // max stack size.
                        // We arguably should check that and return an error here.
                        .unwrap()
                        .with_float_input("x", input)
                        .build();
                    let expected = input * input * input
                        - OrderedFloat::<f64>::from(2f64) * input * input
                        - input;
                    #[allow(clippy::option_if_let_else)]
                    match state.run_to_completion() {
                        Ok(final_state) => final_state
                            .stack::<OrderedFloat<f64>>()
                            .top()
                            .map_or(penalty_value, |answer| (answer - expected).abs().into()),
                        Err(_) => {
                            // Do some logging, perhaps?
                            penalty_value
                        }
                    }
                })
                .collect();
            errors
        },
    );

    let num_test_cases = 10;
    let lexicase = Lexicase::new(num_test_cases);
    let binary_tournament = Tournament::new(2);

    let selector: Weighted<Pop> = Weighted::new(Best, 1)
        .with_selector(lexicase, 5)
        .with_selector(binary_tournament, args.population_size - 1);

    let mut rng = thread_rng();

    let gene_generator = uniform_distribution_of![<PushInstruction>
        FloatInstruction::Add,
        FloatInstruction::Subtract,
        FloatInstruction::Multiply,
        FloatInstruction::ProtectedDivide,
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
