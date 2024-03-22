use anyhow::{Context, Result};
use rand::{rngs::ThreadRng, seq::IndexedRandom};

use super::Selector;
use crate::population::Population;

pub struct Weighted<P: Population> {
    selectors: Vec<(Box<dyn Selector<P> + Send + Sync>, usize)>,
}

impl<P: Population> Weighted<P> {
    // Since we should never have an empty collection of weighted selectors,
    // the `new` implementation takes an initial selector so `selectors` is
    // guaranteed to never be empty.
    #[must_use]
    pub fn new<S>(selector: S, weight: usize) -> Self
    where
        S: Selector<P> + Send + Sync + 'static,
    {
        Self {
            selectors: vec![(Box::new(selector), weight)],
        }
    }

    #[must_use]
    pub fn with_selector<S>(mut self, selector: S, weight: usize) -> Self
    where
        S: Selector<P> + Send + Sync + 'static,
    {
        self.selectors.push((Box::new(selector), weight));
        self
    }
}

impl<P> Selector<P> for Weighted<P>
where
    P: Population,
{
    fn select<'pop>(
        &self,
        population: &'pop P,
        rng: &mut ThreadRng,
    ) -> Result<&'pop P::Individual> {
        let (selector, _) = self
            .selectors
            .choose_weighted(rng, |(_, w)| *w)
            .context("The set of selectors was empty")?;
        selector.select(population, rng)
    }
}
