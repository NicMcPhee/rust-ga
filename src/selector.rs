use rand::rngs::ThreadRng;

use crate::population::Population;

pub mod best;
pub mod lexicase;
pub mod random;
pub mod tournament;
pub mod weighted;

// TODO: Change `Selector` so it acts on a more general collection than `Population`.
//  I think that all we need are some sort of collection or iterator, and then all
//  dependency on `Population` and `Individual` should be able to be removed from
//  this module.
// TODO: Is there a circumstance where selection should fail? If so, do we want to have
//  it return `Option<Individual>` or even `Result<Individual, Error>`? Not sure.
//  esitsu@Twitch suggested, for example, having a selector with a thresh hold and then
//  a composite that keeps trying selectors until it finds one that works.

pub trait Selector<P: Population>: Sync {
    fn select<'pop>(&self, rng: &mut ThreadRng, population: &'pop P) -> &'pop P::Individual;
}

impl<'a, P: Population> Selector<P> for &'a dyn Selector<P> {
    fn select<'pop>(&self, rng: &mut ThreadRng, population: &'pop P) -> &'pop P::Individual {
        (*self).select(rng, population)
    }
}

impl<'a, P: Population> Selector<P> for &'a (dyn Selector<P> + Send) {
    fn select<'pop>(&self, rng: &mut ThreadRng, population: &'pop P) -> &'pop P::Individual {
        (*self).select(rng, population)
    }
}

impl<'a, P: Population> Selector<P> for &'a (dyn Selector<P> + Send + Sync) {
    fn select<'pop>(&self, rng: &mut ThreadRng, population: &'pop P) -> &'pop P::Individual {
        (*self).select(rng, population)
    }
}
