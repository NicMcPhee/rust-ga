use anyhow::{ensure, Context, Result};
use rand::{prelude::IndexedRandom, rngs::ThreadRng};

use super::Selector;
use crate::population::Population;

pub struct Tournament {
    size: usize,
}

impl Tournament {
    #[must_use]
    pub const fn new(size: usize) -> Self {
        Self { size }
    }
}

impl<P> Selector<P> for Tournament
where
    P: Population + AsRef<[P::Individual]>,
    P::Individual: Ord,
{
    fn select<'pop>(
        &self,
        population: &'pop P,
        rng: &mut ThreadRng,
    ) -> Result<&'pop P::Individual> {
        ensure!(
            population.size() >= self.size,
            "The population had size {} and we wanted a tournament of size {}",
            population.size(),
            self.size
        );
        population
            .as_ref()
            .choose_multiple(rng, self.size)
            .max()
            .with_context(|| "The tournament was empty; should have been {size}")
    }
}
