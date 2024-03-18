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
    genome::bitstring::Bitstring, mutator::with_one_over_length::WithOneOverLength,
    recombinator::two_point_xo::TwoPointXo,
};
use rand::{
    distributions::{Distribution, Standard},
    thread_rng,
};

use crate::args::{Args, RunModel};

#[must_use]
pub fn count_ones(bits: &[bool]) -> TestResults<test_results::Score<i64>> {
    bits.iter().map(|bit| i64::from(*bit)).collect()
}

fn main() -> Result<()> {
    // Using `Error` in `TestResults<Error>` will have the run favor smaller
    // values, where using `Score` (e.g., `TestResults<Score>`) will have the run
    // favor larger values.
    type Pop = Vec<EcIndividual<Bitstring, TestResults<test_results::Score<i64>>>>;

    let args = Args::parse();

    let scorer = FnScorer(|bitstring: &Bitstring| count_ones(&bitstring.bits));

    let num_test_cases = args.bit_length;

    let lexicase = Lexicase::new(num_test_cases);
    let binary_tournament = Tournament::new(2);

    let selector: Weighted<Pop> = Weighted::new(Best, 1)
        .with_selector(lexicase, 5)
        .with_selector(binary_tournament, args.population_size - 1);

    let mut rng = thread_rng();

    let population = Standard
        .to_collection_generator(args.bit_length)
        .with_scorer(scorer)
        .into_collection_generator(args.population_size)
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
