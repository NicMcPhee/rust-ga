use rand::{rngs::ThreadRng, seq::SliceRandom};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use crate::{population::{Population, Selector}, individual::Individual};

type ChildMaker<T> = dyn Fn(&mut ThreadRng, &Generation<T>) -> Individual<T> + Send + Sync;

pub struct Generation<'a, T> {
    pub population: Population<T>,
    selectors: &'a Vec<&'a Selector<T>>,
    make_child: &'a ChildMaker<T>
}

impl<'a, T> Generation<'a, T> {
    pub fn new(population: Population<T>, selectors: &'a Vec<&Selector<T>>, make_child: &'a ChildMaker<T>) -> Self {
        assert!(!population.is_empty());
        assert!(!selectors.is_empty());
        Self {
            population,
            selectors,
            make_child
        }
    }

    pub fn best_individual(&self) -> &Individual<T> {
        self.population.best_individual()
    }

    pub fn get_parent(&self, rng: &mut ThreadRng) -> &Individual<T> {
        // The set of selectors should be non-empty, and if it is, then we
        // should be able to safely unwrap the `choose()` call.
        let s = self.selectors.choose(rng).unwrap();
        // The population should be non-empty, and if it is, then we should be
        // able to safely unwrap the selection call.
        s(&self.population).unwrap()
    }
}

impl<'a, T: Send + Sync> Generation<'a, T> {
    pub fn par_next(&self) -> Self {
        let previous_individuals = &self.population.individuals;
        let pop_size = previous_individuals.len();
        let individuals 
            = (0..pop_size)
                .into_par_iter()
                .map(|_| self)
                .map_init(rand::thread_rng, self.make_child)
                .collect();
        Self { 
            population: Population { individuals },
            selectors: self.selectors,
            make_child: self.make_child
        }
    }

    // TODO: Create a `next()` that doesn't use parallelism, i.e., skips the `into_par_iter()` bit.
}