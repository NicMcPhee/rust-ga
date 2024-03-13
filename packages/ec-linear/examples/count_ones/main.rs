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
        recombinator::Recombine,
        selector::{
            best::Best, lexicase::Lexicase, tournament::Tournament, weighted::Weighted, Select,
            Selector,
        },
        Composable,
    },
    test_results::{Score, TestResults},
};
use ec_linear::{
    genome::bitstring::Bitstring, mutator::with_one_over_length::WithOneOverLength,
    recombinator::two_point_xo::TwoPointXo,
};
use rand::{
    distributions::{Distribution, Standard},
    thread_rng,
};

use crate::args::{Args, RunModel};

#[must_use]
pub fn count_ones(bits: &[bool]) -> TestResults<Score<i64>> {
    bits.iter().copied().map(i64::from).collect()
}

fn main() -> Result<()> {
    let Args {
        run_model,
        population_size,
        bit_length,
        num_generations,
    } = Args::parse();

    let mut rng = thread_rng();

    let scorer = FnScorer(|bitstring: &Bitstring| count_ones(&bitstring.bits));

    let num_test_cases = bit_length;

    let selector = Weighted::new(Best, 1)
        .with_selector(Lexicase::new(num_test_cases), 5)
        .with_selector(Tournament::new(2), population_size - 1);

    let population = Standard
        .to_collection_generator(bit_length)
        .with_scorer(scorer)
        .into_collection_generator(population_size)
        .sample(&mut rng);

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

    for generation_number in 0..num_generations {
        match run_model {
            RunModel::Serial => generation.serial_next()?,
            RunModel::Parallel => generation.par_next()?,
        }

        let best = Best.select(generation.population(), &mut rng)?;
        // TODO: Change 2 to be the smallest number of digits needed for
        //  num_generations-1.
        println!("Generation {generation_number:2} best is {best}");
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use ec_core::test_results::{self, TestResults};

    use super::count_ones;

    #[test]
    fn non_empty() {
        let input = [false, true, true, true, false, true];
        let output: TestResults<test_results::Score<i64>> =
            [0, 1, 1, 1, 0, 1].into_iter().collect();
        assert_eq!(output, count_ones(&input));
    }
}
