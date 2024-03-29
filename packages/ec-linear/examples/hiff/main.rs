pub mod args;

use std::{iter::once, ops::Not};

use anyhow::{ensure, Result};
use clap::Parser;
use ec_core::{
    generation::Generation,
    generator::{collection::CollectionGenerator, Generator},
    individual::{
        ec::{self, EcIndividual},
        scorer::FnScorer,
    },
    operator::{
        genome_extractor::GenomeExtractor,
        genome_scorer::GenomeScorer,
        mutator::Mutate,
        recombinator::Recombine,
        selector::{
            best::Best, lexicase::Lexicase, tournament::Tournament, weighted::Weighted, Select,
            Selector,
        },
        Composable,
    },
    test_results::{self, TestResults},
};
use ec_linear::{
    genome::bitstring::{Bitstring, BoolGenerator},
    mutator::with_one_over_length::WithOneOverLength,
    recombinator::two_point_xo::TwoPointXo,
};
use rand::thread_rng;

use crate::args::{Args, RunModel};

#[must_use]
fn hiff(bits: &[bool]) -> (bool, TestResults<test_results::Score<usize>>) {
    let len = bits.len();
    if len < 2 {
        (true, once(test_results::Score::from(len)).collect())
    } else {
        let half_len = len / 2;
        let (left_all_same, left_score) = hiff(&bits[..half_len]);
        let (right_all_same, right_score) = hiff(&bits[half_len..]);
        let all_same = left_all_same && right_all_same && bits[0] == bits[half_len];
        (
            all_same,
            left_score
                .results
                .into_iter()
                .chain(right_score.results)
                .chain(once(test_results::Score::from(if all_same {
                    len
                } else {
                    0
                })))
                .collect(),
        )
    }
}

fn main() -> Result<()> {
    // Using `Error` in `TestResults<Error>` will have the run favor smaller
    // values, where using `Score` (e.g., `TestResults<Score>`) will have the run
    // favor larger values.
    type Pop = Vec<EcIndividual<Bitstring, TestResults<test_results::Score<usize>>>>;

    let args = Args::parse();

    let scorer = FnScorer(|bitstring: &Bitstring| hiff(&bitstring.bits).1);

    let num_test_cases = 2 * args.bit_length - 1;

    let lexicase = Lexicase::new(num_test_cases);
    let binary_tournament = Tournament::new(2);

    let selector: Weighted<Pop> = Weighted::new(Best, 1)
        .with_selector(lexicase, 5)
        .with_selector(binary_tournament, args.population_size - 1);

    let mut rng = thread_rng();

    let boolean_generator = BoolGenerator { p: 0.5 };

    let bitstring_generator = CollectionGenerator {
        size: args.bit_length,
        element_generator: boolean_generator,
    };

    let individual_generator = ec::IndividualGenerator {
        scorer,
        genome_generator: bitstring_generator,
    };

    let population_generator = CollectionGenerator {
        size: args.population_size,
        element_generator: individual_generator,
    };
    let population = population_generator.generate(&mut rng)?;

    ensure!(population.is_empty().not());

    println!("{population:?}");

    // Let's assume the process will be generational, i.e., we replace the entire
    // population with newly created/selected individuals every generation.
    // `generation` will be a mutable operator (containing the data structures for
    // the population(s) and recombinators, scorers, etc.) that acts on a population
    // returning a new population. We'll have different generation operators for
    // serial vs. parallel generation of new individuals.

    let make_new_individual = Select::new(selector)
        .apply_twice()
        .then_map(GenomeExtractor)
        .then(Recombine::new(TwoPointXo))
        .then(Mutate::new(WithOneOverLength))
        .wrap::<GenomeScorer<_, _>>(scorer);

    // generation::new() will take
    //   * a pipeline that gets us from population -> new individual
    //   * an initial population.
    let mut generation = Generation::new(make_new_individual, population);

    // TODO: It might be useful to insert some kind of logging system so we can
    //   make this less imperative in nature.

    (0..args.num_generations).try_for_each(|generation_number| {
        match args.run_model {
            RunModel::Serial => generation.serial_next()?,
            RunModel::Parallel => generation.par_next()?,
        }

        let best = Best.select(generation.population(), &mut rng)?;
        // TODO: Change 2 to be the smallest number of digits needed for
        //  args.num_generations-1.
        println!("Generation {generation_number:2} best is {best}");

        Ok::<(), anyhow::Error>(())
    })?;

    Ok(())
}
