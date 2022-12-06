#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]

use std::iter::Sum;
use std::ops::Not;

use args::{Args, RunModel, TargetProblem};
use individual::Individual;
use rand::rngs::ThreadRng;

use bitstring::{count_ones, hiff, Bitstring, LinearCrossover, LinearMutation};
use child_maker::ChildMaker;
use generation::Generation;
use individual::ec::EcIndividual;
use population::VecPop;
use selector::lexicase::Lexicase;
use selector::Selector;
#[allow(unused_imports)]
use test_results::{Error, Score, TestResults};

use crate::selector::best::Best;
use crate::selector::tournament::Tournament;
use crate::selector::weighted::Weighted;

pub mod args;
pub mod bitstring;
pub mod child_maker;
pub mod generation;
pub mod individual;
pub mod population;
pub mod selector;
pub mod test_results;

/// # Panics
///
/// This can panic for a whole host of reasons, mostly because the
/// population or the collection of selectors is empty.
pub fn do_main(args: Args) {
    let scorer = match args.target_problem {
        TargetProblem::CountOnes => count_ones,
        TargetProblem::Hiff => hiff,
    };

    let num_test_cases = match args.target_problem {
        TargetProblem::CountOnes => args.bit_length,
        TargetProblem::Hiff => 2 * args.bit_length - 1,
    };

    let lexicase = Lexicase::new(num_test_cases);
    let binary_tournament = Tournament::new(2);
    let best = Best {};

    let selector = Weighted::new(&best, 1)
        .with_selector(&lexicase, 5)
        .with_selector(&binary_tournament, args.population_size - 1);

    let population = VecPop::new_bitstring_population(
        args.population_size,
        args.bit_length,
        // TODO: I should really have a function somewhere that converts functions
        //   that return vectors of scores to `TestResults` structs.
        |bitstring| scorer(bitstring).into_iter().map(From::from).sum(),
    );
    assert!(population.is_empty().not());

    // TODO: We probably want `scorer` to be generating the `TestResults` values
    //   and have it be "in charge" of whether we're using `Score` or `Error`. Then
    //   the child maker shouldn't need to care and we can just use `TestResults<R>` here.
    let child_maker = TwoPointXoMutateChildMaker::new(&scorer);

    // Using `Error` in `TestResults<Error>` will have the run favor smaller
    // values, where using `Score` (e.g., `TestResults<Score>`) will have the run
    // favor larger values.
    let mut generation: Generation<Bitstring, TestResults<Error>> =
        Generation::new(population, &selector, &child_maker);

    assert!(generation.population.is_empty().not());
    // let best = generation.best_individual();
    // println!("{}", best);
    // println!("Pop size = {}", generation.population.size());
    // println!("Bit length = {}", best.genome.len());

    (0..args.num_generations).for_each(|generation_number| {
        generation = match args.run_model {
            RunModel::Serial => generation.next(),
            RunModel::Parallel => generation.par_next(),
        };
        let best = generation.best_individual();
        // TODO: Change 2 to be the smallest number of digits needed for
        //  args.num_generations-1.
        println!("Generation {:2} best is {}", generation_number, best);
    });
}

struct TwoPointXoMutateChildMaker<'a> {
    scorer: &'a (dyn Fn(&[bool]) -> Vec<i64> + Sync),
}

impl<'a> TwoPointXoMutateChildMaker<'a> {
    fn new(scorer: &(dyn Fn(&[bool]) -> Vec<i64> + Sync)) -> TwoPointXoMutateChildMaker {
        TwoPointXoMutateChildMaker { scorer }
    }
}

impl<'a, R> ChildMaker<EcIndividual<Bitstring, TestResults<R>>> for TwoPointXoMutateChildMaker<'a>
where
    R: Sum + Copy + From<i64>,
{
    fn make_child(
        &self,
        rng: &mut ThreadRng,
        population: &VecPop<EcIndividual<Bitstring, TestResults<R>>>,
        selector: &dyn Selector<EcIndividual<Bitstring, TestResults<R>>>,
    ) -> EcIndividual<Bitstring, TestResults<R>> {
        let first_parent = selector.select(rng, population);
        let second_parent = selector.select(rng, population);

        let genome = first_parent
            .genome()
            .two_point_xo(second_parent.genome(), rng)
            .mutate_one_over_length(rng);
        let test_results = (self.scorer)(&genome).into_iter().map(From::from).sum();
        EcIndividual::new(genome, test_results)
    }
}
