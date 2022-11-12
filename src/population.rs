#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![allow(clippy::missing_panics_doc)]

use std::{borrow::Borrow, mem::swap, num};
use rand::seq::SliceRandom;

use rand::rngs::ThreadRng;
use rayon::prelude::{ParallelExtend, IntoParallelIterator, ParallelIterator};

use crate::individual::{Individual, TestResults};

pub struct Population<'a, G, R> {
    pub individuals: Vec<Individual<G, R>>,
    pub lexicase_sortings: Vec<Vec<&'a Individual<G, R>>>,
}

pub fn compute_lexicase_sortings<'a, G: IntoIterator, R: Ord>(individuals: &Vec<Individual<G, TestResults<R>>>) -> Vec<Vec<&'a Individual<G, TestResults<R>>>> {
    assert!(individuals.len() > 0, "The collection of individuals was empty");
    let g = individuals[0].genome.into_iter();
    let num_test_cases = g.count();
    (0..num_test_cases).map(|test_index| {
        let refs: Vec<_> = individuals.iter().collect();
        refs.sort_by_key(|i| {
            i.test_results.results[test_index]
        });
        refs
    }).collect()
}

impl<'a, G: Send, R: Send> Population<'a, G, R> {
    /*
     * See the lengthy comment in `individual.rs` on why we need the
     * whole `Borrow<H>` business.
     */
    pub fn new<H>(
            pop_size: usize,
            make_genome: impl Fn(&mut ThreadRng) -> G + Send + Sync, 
            run_tests: impl Fn(&H) -> R + Send + Sync) 
        -> Self
    where
        G: Borrow<H>,
        H: ?Sized
    {
        let mut individuals = Vec::with_capacity(pop_size);
        individuals.par_extend((0..pop_size)
            .into_par_iter()
            .map_init(
                rand::thread_rng,
                |rng, _| {
                    Individual::new(&make_genome, &run_tests, rng)
                })
        );
        let lexicase_sortings = compute_lexicase_sortings(&individuals); 
        Self {
            individuals,
            lexicase_sortings,
        }
    }
}

impl<'a, G, R> Population<'a, G, R> {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.individuals.is_empty()
    }

    #[must_use]
    pub fn size(&self) -> usize {
        self.individuals.len()
    }
}

impl<'a, G: Eq, R: Ord> Population<'a, G, R> {
    /// # Panics
    /// 
    /// Will panic if the population is empty.
    #[must_use]
    pub fn best_individual(&self) -> &Individual<G, R> {
        assert!(!self.individuals.is_empty());
        #[allow(clippy::unwrap_used)]
        self
            .individuals
            .iter()
            .max()
            .unwrap()
    }
}

impl<'a, G, R> Population<'a, G, R> {
    #[must_use]
    pub fn random(&self) -> Option<&Individual<G, R>> {
        self.individuals.choose(&mut rand::thread_rng())
    }
}

impl<'a, G: Eq, R: Ord> Population<'a, G, R> {
    pub fn make_tournament_selector(tournament_size: usize) -> impl Fn(&'a Population<'a, G, R>) -> &'a Individual<G, R> {
        move |pop: &Self| {
            pop.tournament(tournament_size)
        }
    }

    #[must_use]
    pub fn tournament(&self, tournament_size: usize) -> &Individual<G, R> {
        assert!(self.individuals.len()>=tournament_size && tournament_size>0);
        // Since we know that the population and tournament aren't empty, we
        // can safely unwrap() the `.max()` call.
        #[allow(clippy::unwrap_used)]
        self.individuals
            .choose_multiple(&mut rand::thread_rng(), tournament_size)
            .max()
            .unwrap()
    }
}

impl<'a, G, R: PartialOrd> Population<'a, G, TestResults<R>> {
    #[must_use]
    pub fn lexicase(&self) -> &Individual<G, TestResults<R>> {
        // Candidate set is initially the whole population.
        // Shuffle the (indices of the) test cases.
        // For each test in turn:
        //   * Find the best score of any individual still in
        //     in the candidate set on that test case.
        //   * Remove any individual from the candidate set that
        //     is worse than that best score on that test case.
        // Go until you get to a single individual or you run
        // out of test cases.

        // TODO: Compute these bits once when the population is initially constructed
        //   and then just look it up when necessary instead of recomputing it
        //   for every selection.
        let first_individual = &self.individuals[0];
        let first_results = &first_individual.test_results.results;
        let num_results = first_results.len();
        let mut case_indices: Vec<usize> = (0..num_results).collect();

        case_indices.shuffle(&mut rand::thread_rng());

        let mut candidates: Vec<_> = self.individuals
            .iter()
            .collect();

        let mut winners = Vec::with_capacity(candidates.len());
        for test_case_index in case_indices {
            assert!(!candidates.is_empty(), "The set of lexicase candidates shouldn't be empty");
            if candidates.len() == 1 {
                break;
            }
            winners.clear();
            winners.push(candidates[0]);
            for c in &candidates[1..] {
                if c.test_results.results[test_case_index] > winners[0].test_results.results[test_case_index] {
                    winners.clear();
                    winners.push(c);
                } else if c.test_results.results[test_case_index] == winners[0].test_results.results[test_case_index] {
                    winners.push(c);
                }
            }
            swap(&mut candidates, &mut winners);
        }

        candidates.shuffle(&mut rand::thread_rng());
        candidates[0]
    }

    #[must_use]
    pub fn sorting_lexicase(&self) -> &Individual<G, TestResults<R>> {
        todo!()
    }
}
