use std::ops::Not;

use anyhow::{Context, Result};
use ec_core::operator::mutator::Mutator;
use num_traits::ToPrimitive;
use rand::rngs::ThreadRng;

use super::with_rate::WithRate;
use crate::genome::Linear;

pub struct WithOneOverLength;

impl<T> Mutator<Vec<T>> for WithOneOverLength
where
    T: Not<Output = T>,
{
    fn mutate(&self, genome: Vec<T>, rng: &mut ThreadRng) -> Result<Vec<T>> {
        let genome_length = genome.len().to_f32().with_context(|| {
            format!(
                "The genome length {} couldn't be converted to an f32 value",
                genome.len()
            )
        })?;
        let mutation_rate = 1.0 / genome_length;
        let mutator = WithRate::new(mutation_rate);
        mutator.mutate(genome, rng)
    }
}

impl<T> Mutator<T> for WithOneOverLength
where
    T: Linear + FromIterator<T::Gene> + IntoIterator<Item = T::Gene>,
    T::Gene: Not<Output = T::Gene>,
{
    fn mutate(&self, genome: T, rng: &mut ThreadRng) -> Result<T> {
        let genome_length = genome.size().to_f32().with_context(|| {
            format!(
                "The genome length {} couldn't be converted to an f32 value",
                genome.size()
            )
        })?;
        let mutation_rate = 1.0 / genome_length;
        let mutator = WithRate::new(mutation_rate);
        mutator.mutate(genome, rng)
    }
}

#[cfg(test)]
mod tests {
    use std::iter::zip;

    use ec_core::operator::mutator::Mutator;

    use crate::{genome::bitstring::Bitstring, mutator::with_one_over_length::WithOneOverLength};

    // This test is stochastic, so I'm going to ignore it most of the time.
    #[test]
    #[ignore]
    #[allow(clippy::unwrap_used)]
    fn mutate_one_over_does_not_change_much() {
        let mut rng = rand::thread_rng();
        let num_bits = 100;
        let parent_bits: Bitstring = Bitstring::random(num_bits, &mut rng);

        let child_bits = WithOneOverLength
            .mutate(parent_bits.clone(), &mut rng)
            .unwrap();

        let num_differences = zip(parent_bits, child_bits)
            .filter(|(p, c)| *p != *c)
            .count();
        println!("Num differences = {num_differences}");
        assert!(
            0 < num_differences,
            "We're expecting at least one difference"
        );
        assert!(
            num_differences < num_bits / 10,
            "We're not expecting lots of differences, and got {num_differences}."
        );
    }
}
