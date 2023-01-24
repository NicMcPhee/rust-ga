use std::iter::Sum;

use rand::rngs::ThreadRng;

use crate::{individual::{ec::EcIndividual, Individual}, bitstring::{Bitstring, LinearCrossover, LinearMutation}, test_results::TestResults, selector::Selector};

use super::ChildMaker;

#[derive(Clone)]
pub struct TwoPointXoMutate<'a> {
    pub scorer: &'a (dyn Fn(&[bool]) -> Vec<i64> + Sync),
}

impl<'a> TwoPointXoMutate<'a> {
    pub fn new(scorer: &(dyn Fn(&[bool]) -> Vec<i64> + Sync)) -> TwoPointXoMutate {
        TwoPointXoMutate { scorer }
    }
}

impl<'a, S, R> ChildMaker<Vec<EcIndividual<Bitstring, TestResults<R>>>, S>
    for TwoPointXoMutate<'a>
where
    S: Selector<Vec<EcIndividual<Bitstring, TestResults<R>>>>,
    R: Sum + Copy + From<i64>,
{
    fn make_child(
        &self,
        rng: &mut ThreadRng,
        population: &Vec<EcIndividual<Bitstring, TestResults<R>>>,
        selector: &S,
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