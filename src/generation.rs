use rand::{rngs::ThreadRng, seq::SliceRandom};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use crate::{individual::Individual, population::Population};

pub type Selector<T> = dyn Fn(&Population<T>) -> Option<&Individual<T>> + Sync + Send;
pub type WeightedSelector<'a, T> = (&'a Selector<T>, usize);
pub type ChildMaker<T> = dyn Fn(&mut ThreadRng, &Generation<T>) -> Individual<T> + Send + Sync;

// TODO: Extend the vector of Selectors to a WeightedParentSelector that is essentially
//   a wrapper around `rand::distributions::WeightedChoice` so we can
//   provide weights on the different selectors.
// TODO: Should the `scorer` be inside the generation so we don't have to keep
//   capturing it and passing it around?
// TODO: Should there actually be a `Run` type (or a `RunParams` type) that
//   holds all this stuff and is used to make them available to types like
//   `Generation` and `Population`?
pub struct Generation<'a, T> {
    pub population: Population<T>,
    weighted_selectors: &'a Vec<WeightedSelector<'a, T>>,
    make_child: &'a ChildMaker<T>,
}

impl<'a, T> Generation<'a, T> {
    pub fn new(
        population: Population<T>,
        weighted_selectors: &'a Vec<WeightedSelector<'a, T>>,
        make_child: &'a ChildMaker<T>,
    ) -> Self {
        assert!(!population.is_empty());
        assert!(!weighted_selectors.is_empty());
        Self {
            population,
            weighted_selectors,
            make_child,
        }
    }

    pub fn best_individual(&self) -> &Individual<T> {
        self.population.best_individual()
    }

    pub fn get_parent(&self, rng: &mut ThreadRng) -> &Individual<T> {
        // The set of selectors should be non-empty, and if it is, then we
        // should be able to safely unwrap the `choose()` call.
        let s = self
            .weighted_selectors
            .choose_weighted(rng, |item| item.1)
            .unwrap()
            .0;
        // choices.choose_weighted(&mut rng, |item| item.1).unwrap().0)

        // The population should be non-empty, and if it is, then we should be
        // able to safely unwrap the selection call.
        s(&self.population).unwrap()
    }
}

impl<'a, T: Send + Sync> Generation<'a, T> {
    /// Make the next generation using a Rayon parallel iterator.
    pub fn par_next(&self) -> Self {
        let previous_individuals = &self.population.individuals;
        let pop_size = previous_individuals.len();
        let individuals = (0..pop_size)
            .into_par_iter()
            // "Convert" the individual number (which we never use) into
            // the current `Generation` object so the `make_child` closure
            // will have access to the selectors and population.
            .map(|_| self)
            .map_init(rand::thread_rng, self.make_child)
            .collect();
        Self {
            population: Population { individuals },
            weighted_selectors: self.weighted_selectors,
            make_child: self.make_child,
        }
    }

    /// Make the next generation serially.
    pub fn next(&self) -> Self {
        let previous_individuals = &self.population.individuals;
        let pop_size = previous_individuals.len();
        let mut rng = rand::thread_rng();
        let individuals = (0..pop_size)
            .map(|_| (self.make_child)(&mut rng, self))
            .collect();
        Self {
            population: Population { individuals },
            weighted_selectors: self.weighted_selectors,
            make_child: self.make_child,
        }
    }
}
