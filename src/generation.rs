use rand::{rngs::ThreadRng, seq::SliceRandom};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use crate::{population::{Population, compute_lexicase_sortings}, individual::{Individual, TestResults}};

pub type Selector<G, R> = dyn for<'a> Fn(&Population<'a, G, R>) -> &'a Individual<G, R> + Sync + Send;
pub type WeightedSelector<G, R> = (&'static Selector<G, R>, usize);
pub type ChildMaker<G, R> = dyn Fn(&mut ThreadRng, &Generation<G, R>) -> Individual<G, R> + Send + Sync;

// TODO: Extend the vector of Selectors to a WeightedParentSelector that is essentially
//   a wrapper around `rand::distributions::WeightedChoice` so we can
//   provide weights on the different selectors.
// TODO: Should the `scorer` be inside the generation so we don't have to keep
//   capturing it and passing it around?
// TODO: Should there actually be a `Run` type (or a `RunParams` type) that
//   holds all this stuff and is used to make them available to types like
//   `Generation` and `Population`?
pub struct Generation<'a, G: 'static, R: 'static> {
    pub population: Population<'a, G, R>,
    // selector: &'a Selector<G, R>,
    // weighted_selectors: &'static Vec<WeightedSelector<G, R>>,
    make_child: &'a ChildMaker<G, R>
}

impl<'a, G: Eq, R: Ord> Generation<'a, G, R> {
    /// # Panics
    ///
    /// This can panic if the population is empty or the weighted set of
    /// selectors is empty.
    // pub fn new(population: Population<'a, G, R>, weighted_selectors: &'static Vec<WeightedSelector<G, R>>, make_child: &'a ChildMaker<G, R>) -> Self {
    pub fn new(population: Population<'a, G, R>, make_child: &'a ChildMaker<G, R>) -> Self {
            assert!(!population.is_empty());
        // assert!(!weighted_selectors.is_empty());
        Self {
            population,
            // selector,
            // weighted_selectors,
            make_child
        }
    }

    #[must_use]
    pub fn best_individual(&self) -> &Individual<G, R> {
        self.population.best_individual()
    }
}

impl<'a, G, R: PartialOrd> Generation<'a, G, TestResults<R>> {
    /// # Panics
    /// 
    /// This can panic if the set of selectors is empty.
    pub fn get_parent(&self, rng: &mut ThreadRng) -> &Individual<G, TestResults<R>> {
        // The set of selectors should be non-empty, and if it is, then we
        // should be able to safely unwrap the `choose()` call.
        #[allow(clippy::unwrap_used)]
        // let s 
        //     = self.weighted_selectors.choose_weighted(rng, |item| item.1).unwrap().0;
        let pop = &self.population;
        pop.lexicase()
        // (self.selector)(&self.population)
    }
}

impl<'a, G: Send + Sync, R: Send + Sync> Generation<'a, G, R> {
    /// Make the next generation using a Rayon parallel iterator.
    #[must_use]
    pub fn par_next(&self) -> Self {
        let previous_individuals = &self.population.individuals;
        let pop_size = previous_individuals.len();
        let individuals 
            = (0..pop_size)
                .into_par_iter()
                // "Convert" the individual number (which we never use) into
                // the current `Generation` object so the `make_child` closure
                // will have access to the selectors and population.
                .map(|_| self)
                .map_init(rand::thread_rng, self.make_child)
                .collect();
        let lexicase_sortings = compute_lexicase_sortings(&individuals);
        Self { 
            population: Population { individuals, lexicase_sortings },
            // selector: self.selector,
            // weighted_selectors: self.weighted_selectors,
            make_child: self.make_child
        }
    }

    /// Make the next generation serially.
    #[must_use]
    pub fn next(&self) -> Self {
        let previous_individuals = &self.population.individuals;
        let pop_size = previous_individuals.len();
        let mut rng = rand::thread_rng();
        let individuals 
            = (0..pop_size)
                .map(|_| (self.make_child)(&mut rng, self))
                .collect();
        let lexicase_sortings = compute_lexicase_sortings(&individuals);
        Self { 
            population: Population { individuals, lexicase_sortings },
            // selector: self.selector,
            // weighted_selectors: self.weighted_selectors,
            make_child: self.make_child
        }
    }
}