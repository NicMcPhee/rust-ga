pub mod args;

use std::ops::Not;

use anyhow::{ensure, Result};
use clap::Parser;
use ec_core::{
    generation::Generation,
    generator::{collection::ConvertToCollectionGenerator, Generator},
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
};
use ec_linear::mutator::umad::Umad;
use push::{
    genome::plushy::{GeneGenerator, Plushy},
    instruction::{variable_name::VariableName, BoolInstruction, IntInstruction, PushInstruction},
    push_vm::{program::PushProgram, push_state::PushState, HasStack, State},
    vec_into,
};
use rand::{thread_rng, RngCore};

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
    type Pop = Vec<EcIndividual<Plushy, TestResults<test_results::Error<i64>>>>;
    // The penalty value to use when an evolved program doesn't have an expected
    // "return" value on the appropriate stack at the end of its execution.
    let penalty_value: i64 = 1_000;

    let args = Args::parse();

    let mut rng = thread_rng();
    // Inputs from -4 (inclusive) to 4 (exclusive) in increments of 0.25.
    let training_inputs: Vec<(i64, i64, i64, i64)> = (0..100)
        .map(|_| {
            (
                (rng.next_u32() % 100).into(),
                (rng.next_u32() % 100).into(),
                (rng.next_u32() % 100).into(),
                (rng.next_u32() % 100).into(),
            )
        })
        .collect::<Vec<_>>();

    /*
     * The `scorer` will need to take an evolved program (sequence of
     * instructions) and run it on all the inputs in `training_inputs`, and
     * then return a `TestResults` object that contains the results of running
     * that program minus the expected results.
     */
    let scorer = FnScorer(|genome: &Plushy| -> TestResults<test_results::Error<i64>> {
        let program = Vec::<PushProgram>::from(genome.clone());
        let errors: TestResults<test_results::Error<i64>> = training_inputs
            .iter()
            .map(|&(a, b, c, d)| {
                #[allow(clippy::unwrap_used)]
                let state = PushState::builder()
                    .with_max_stack_size(1000)
                    .with_program(program.clone())
                    // This will return an error if the program is longer than the allowed
                    // max stack size.
                    // We arguably should check that and return an error here.
                    .unwrap()
                    .with_int_input("a", a)
                    .with_int_input("b", b)
                    .with_int_input("c", c)
                    .with_int_input("d", d)
                    .build();
                #[allow(clippy::unwrap_used)]
                let expected: i64 = [a, b, c, d].into_iter().min().unwrap();
                #[allow(clippy::option_if_let_else)]
                match state.run_to_completion() {
                    Ok(final_state) => final_state
                        .stack::<i64>()
                        .top()
                        .map_or(penalty_value, |answer| (answer - expected).abs()),
                    Err(_) => {
                        // Do some logging, perhaps?
                        penalty_value
                    }
                }
            })
            .collect();
        errors
    });

    let num_test_cases = training_inputs.len();
    let lexicase = Lexicase::new(num_test_cases);
    let binary_tournament = Tournament::new(2);

    let selector: Weighted<Pop> = Weighted::new(Best, 1)
        .with_selector(lexicase, 5)
        .with_selector(binary_tournament, args.population_size - 1);

    let mut rng = thread_rng();

    let instruction_set = instructions();

    let gene_generator = GeneGenerator::with_uniform_close_probability(instruction_set);

    let population = gene_generator
        .to_collection_generator(args.max_initial_instructions)
        .with_scorer(scorer)
        .into_collection_generator(args.population_size)
        .generate(&mut rng)?;

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

        if best.test_results.total_result.error == 0 {
            println!("SUCCESS");
            break;
        }
    }

    Ok(())
}

fn instructions() -> Vec<PushInstruction> {
    // Use `strum` to generate this list of instructions.
    vec_into![
        IntInstruction::Negate,
        IntInstruction::Abs,
        IntInstruction::Min,
        IntInstruction::Max,
        IntInstruction::Inc,
        IntInstruction::Dec,
        IntInstruction::Add,
        IntInstruction::Subtract,
        IntInstruction::Multiply,
        IntInstruction::ProtectedDivide,
        IntInstruction::Mod,
        IntInstruction::Power,
        IntInstruction::Square,
        IntInstruction::IsZero,
        IntInstruction::IsPositive,
        IntInstruction::IsNegative,
        IntInstruction::IsEven,
        IntInstruction::IsOdd,
        IntInstruction::Equal,
        IntInstruction::NotEqual,
        IntInstruction::LessThan,
        IntInstruction::LessThanEqual,
        IntInstruction::GreaterThan,
        IntInstruction::GreaterThanEqual,
        IntInstruction::FromBoolean,
        BoolInstruction::Not,
        BoolInstruction::Or,
        BoolInstruction::And,
        BoolInstruction::Xor,
        BoolInstruction::Implies,
        BoolInstruction::FromInt,
        // ExecInstruction::IfElse,
        VariableName::from("a"),
        VariableName::from("b"),
        VariableName::from("c"),
        VariableName::from("d"),
    ]
}
