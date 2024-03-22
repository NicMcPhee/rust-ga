use anyhow::{Context, Result};
use rand::{prelude::IndexedRandom, rngs::ThreadRng};

use super::Selector;
use crate::population::Population;

pub struct Random;

impl<P> Selector<P> for Random
where
    P: Population + AsRef<[P::Individual]>,
{
    fn select<'pop>(
        &self,
        population: &'pop P,
        rng: &mut ThreadRng,
    ) -> Result<&'pop P::Individual> {
        population
            .as_ref()
            .choose(rng)
            .context("The population was empty")
    }
}
