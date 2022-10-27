#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]

use std::{borrow::Borrow, mem::swap, cmp::Ordering};
use itertools::Itertools;
use rand::seq::SliceRandom;

use rand::rngs::ThreadRng;
use rayon::prelude::{ParallelExtend, IntoParallelIterator, ParallelIterator};

use crate::individual::Individual;

pub struct Population<T> {
    pub individuals: Vec<Individual<T>>,
}

impl<T: Send> Population<T> {
    /*
     * See the lengthy comment in `individual.rs` on why we need the
     * whole `Borrow<R>` business.
     */
    pub fn new<R>(
            pop_size: usize,
            make_genome: impl Fn(&mut ThreadRng) -> T + Send + Sync, 
            compute_score: impl Fn(&R) -> Vec<i64> + Send + Sync) 
        -> Self
    where
        T: Borrow<R>,
        R: ?Sized
    {
        let mut individuals = Vec::with_capacity(pop_size);
        individuals.par_extend((0..pop_size)
            .into_par_iter()
            .map_init(
                rand::thread_rng,
                |rng, _| {
                    Individual::new(&make_genome, &compute_score, rng)
                })
        );
        Self {
            individuals,
        }
    }
}

impl<T> Population<T> {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.individuals.is_empty()
    }

    #[must_use]
    pub fn size(&self) -> usize {
        self.individuals.len()
    }

    /// # Panics
    /// 
    /// Will panic if the population is empty.
    #[must_use]
    pub fn best_individual(&self) -> &Individual<T> {
        assert!(!self.individuals.is_empty());
        #[allow(clippy::unwrap_used)]
        self
            .individuals
            .iter()
            .max_by_key(
                |ind| ind.total_score
            )
            .unwrap()
    }
}

impl<T> Population<T> {
    /// # Panics
    ///
    /// This will panic if the population is empty.
    #[must_use]
    pub fn best_score(&self) -> Option<&Individual<T>> {
        assert!(!self.individuals.is_empty());
        self.individuals.iter().max_by_key(|ind| ind.total_score)
    }

    #[must_use]
    pub fn random(&self) -> Option<&Individual<T>> {
        self.individuals.choose(&mut rand::thread_rng())
    }

    pub fn make_tournament_selector(tournament_size: usize) -> impl Fn(&Self) -> Option<&Individual<T>> {
        move |pop: &Self| {
            pop.tournament(tournament_size)
        }
    }

    #[must_use]
    pub fn tournament(&self, tournament_size: usize) -> Option<&Individual<T>> {
        self.individuals
            .choose_multiple(&mut rand::thread_rng(), tournament_size)
            .max_by_key(|ind| ind.total_score)
    }

    // TODO: Why are we returning `Option`s for all the selectors?
    pub fn simple_lexicase(&self) -> Option<&Individual<T>> {
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
    
        // Get the number of scores from the first individual.
        let num_scores = match self.individuals.first() {
            Some(ind) => ind.scores.len(),
            None => return None,
        };
    
        // The case indices are the number of scores in a random order.
        let mut case_indices = (0..num_scores).collect::<Vec<_>>();
    
        let mut rng = rand::thread_rng();
        case_indices.shuffle(&mut rng);
    
        let mut candidates = self.individuals.iter().collect::<Vec<_>>();
    
        // Run until there is exactly one candidate.
        while candidates.len() > 1 {
            let idx = match case_indices.pop() {
                Some(i) => i,
                None => break,
            };
    
            let mut best_score = 0;
            candidates.retain(|ind| {
                let score = ind.scores[idx];
                match score.cmp(&best_score) {
                    Ordering::Greater => {
                        best_score = score;
                        true
                    }
                    Ordering::Equal => true,
                    Ordering::Less => false,
                }
            });
        }
    
        candidates.shuffle(&mut rng);
        Some(&candidates[0])
    }

    pub fn lexicase_with_dup_removal(&self) -> Option<&Individual<T>> {
        // Candidate set is initially the whole population.
        // Shuffle the (indices of the) test cases.
        // For each test in turn:
        //   * Find the best score of any individual still in
        //     in the candidate set on that test case.
        //   * Remove any individual from the candidate set that
        //     is worse than that best score on that test case.
        // Go until you get to a single individual or you run
        // out of test cases.

        let first_individual = &self.individuals[0];
        let first_scores = &first_individual.scores;
        let num_scores = first_scores.len();
        let mut case_indices: Vec<usize> = (0..num_scores).collect();

        case_indices.shuffle(&mut rand::thread_rng());

        let mut candidates: Vec<&Individual<T>> = self.individuals
            .iter()
            .unique_by(|ind| &ind.scores)
            .collect();

        for test_case_index in case_indices {
            assert!(!candidates.is_empty(), "The set of lexicase candidates shouldn't be empty");
            if candidates.len() == 1 {
                break;
            }
            let max_score = candidates.iter().max_by_key(|ind| {
                ind.scores[test_case_index]
            }).unwrap().scores[test_case_index];
            candidates = candidates.into_iter().filter(|ind| {
                ind.scores[test_case_index] == max_score
            }).collect();
        }

        candidates.shuffle(&mut rand::thread_rng());
        Some(candidates[0])
    }

    pub fn one_pass_lexicase(&self) -> Option<&Individual<T>> {
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
        let first_scores = &first_individual.scores;
        let num_scores = first_scores.len();
        let mut case_indices: Vec<usize> = (0..num_scores).collect();

        case_indices.shuffle(&mut rand::thread_rng());

        let mut candidates: Vec<&Individual<T>> = self.individuals
            .iter()
            .collect();

        for test_case_index in case_indices {
            assert!(!candidates.is_empty(), "The set of lexicase candidates shouldn't be empty");
            if candidates.len() == 1 {
                break;
            }
            // let mut winners = vec![candidates[0]];
            // for c in &candidates[1..] {
            //     if c.scores[test_case_index] > winners[0].scores[test_case_index] {
            //         winners = Vec::with_capacity(candidates.len());
            //         winners.push(c);
            //     } else if c.scores[test_case_index] == winners[0].scores[test_case_index] {
            //         winners.push(c);
            //     }
            // }
            // candidates = winners;
            let mut winners = Vec::with_capacity(candidates.len());
            winners.push(candidates[0]);
            candidates = candidates
                .into_iter()
                .skip(1)
                .fold(winners, |mut winners, individual| {
                    if individual.scores[test_case_index] > winners[0].scores[test_case_index] {
                        winners.clear();
                        winners.push(individual);
                    } else if individual.scores[test_case_index] == winners[0].scores[test_case_index] {
                        winners.push(individual);
                    }
                    winners
                });
        }

        candidates.shuffle(&mut rand::thread_rng());
        Some(candidates[0])
    }

    pub fn reuse_vector_lexicase(&self) -> Option<&Individual<T>> {
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
        let first_scores = &first_individual.scores;
        let num_scores = first_scores.len();
        let mut case_indices: Vec<usize> = (0..num_scores).collect();

        case_indices.shuffle(&mut rand::thread_rng());

        let mut candidates: Vec<&Individual<T>> = self.individuals
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
                if c.scores[test_case_index] > winners[0].scores[test_case_index] {
                    winners.clear();
                    winners.push(c);
                } else if c.scores[test_case_index] == winners[0].scores[test_case_index] {
                    winners.push(c);
                }
            }
            swap(&mut candidates, &mut winners);
        }

        candidates.shuffle(&mut rand::thread_rng());
        Some(candidates[0])
    }
}
