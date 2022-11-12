#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]

use args::{RunModel, TargetProblem, Args};
use rand::rngs::ThreadRng;

use bitstring::{Bitstring, LinearCrossover, LinearMutation, count_ones, hiff};
use population::Population;
use generation::{Generation, WeightedSelector}; 
use individual::{Individual, TestResults};

use crate::individual::Error;

pub mod args;
pub mod individual;
pub mod population;
pub mod generation;
pub mod bitstring;

/// # Panics
/// 
/// This can panic for a whole host of reasons, mostly because the
/// population or the collection of selectors is empty.
pub fn do_main(args: Args) {
    let scorer = match args.target_problem {
        TargetProblem::CountOnes => count_ones,
        TargetProblem::Hiff => hiff
    };

    // Use lexicase selection almost exclusively, but typically carry forward
    // at least one copy of the best individual (as measured by total fitness).
    // let weighted_selectors = // : Vec<WeightedSelector<Bitstring, TestResults<Error>>> =
    //     vec![
    //             // (&Population::best_individual, 1),
    //             (&Population::lexicase, args.population_size-1)
    //         ];

    // pub type Selector<G, R> = dyn for<'a> Fn(&Population<'a, G, R>) -> &'a Individual<G, R> + Sync + Send;
    // let selector: & dyn for<'r, 'a> Fn(&Population<'a, Bitstring, TestResults<individual::Error>>) -> &'a Individual<Bitstring, TestResults<individual::Error>> = &Population::lexicase;

    let population
        = Population::new_bitstring_population(
            args.population_size, 
            args.bit_length, 
            // TODO: I should really have a function somewhere that converts functions
            //   that return vectors of scores to `TestResults` structs.
            |bitstring| {
                let results = scorer(bitstring);
                let total_result = Error { error: results.iter().sum() };
                TestResults {
                    total_result,
                    results: results.iter().map(|r| Error{ error: *r }).collect()
                }
            });
    assert!(!population.is_empty());

    let make_child = move |rng: &mut ThreadRng, generation: &Generation<Bitstring, TestResults<Error>>| {
        make_child(scorer, rng, generation)
    };

    let mut generation = Generation::new(
        population,
        // &Population::lexicase,
        // &weighted_selectors,
        &make_child
    );

    assert!(!generation.population.is_empty());
    // let best = generation.best_individual();
    // println!("{}", best);
    // println!("Pop size = {}", generation.population.size());
    // println!("Bit length = {}", best.genome.len());

    (0..args.num_generations).for_each(|generation_number| {
        generation = match args.run_model {
            RunModel::Serial => generation.next(),
            RunModel::Parallel => generation.par_next()
        };
        let best = generation.best_individual();
        // TODO: Change 2 to be the smallest number of digits needed for
        //  args.num_generations-1.
        println!("Generation {:2} best is {}", generation_number, best);
    });
}

fn make_child(scorer: impl Fn(&[bool]) -> Vec<i64>,
              rng: &mut ThreadRng, 
              generation: &Generation<Bitstring, TestResults<Error>>) -> Individual<Bitstring, TestResults<Error>> {
    let first_parent = generation.get_parent(rng);
    let second_parent = generation.get_parent(rng);

    let genome
        = first_parent.genome
            .two_point_xo(&second_parent.genome, rng)
            .mutate_one_over_length(rng);
    let results = scorer(&genome);
    let total_result = results.iter().sum();
    Individual { 
        genome,
        test_results: TestResults { total_result: Error { error: total_result }, results: results.iter().map(|r| Error { error: *r }).collect() }
    }
}