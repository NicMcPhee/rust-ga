use anyhow::{ensure, Result};
use ec_core::operator::recombinator::Recombinator;
use rand::{rngs::ThreadRng, Rng};

use super::crossover::Crossover;

pub struct UniformXo;

// TODO: We should get rid of the `Vec<T>` versions when
//   we've completed the migration to a struct-based `Bitstring`.
impl<T: Clone> Recombinator<[Vec<T>; 2]> for UniformXo {
    type Output = Vec<T>;

    fn recombine(
        &self,
        [first_genome, second_genome]: [Vec<T>; 2],
        rng: &mut ThreadRng,
    ) -> Result<Self::Output> {
        ensure!(
            first_genome.len() == second_genome.len(),
            "Attempted to perform UniformXo on genomes of different length: {} and {}",
            first_genome.len(),
            second_genome.len()
        );
        let len = first_genome.len();
        Ok((0..len)
            .map(|pos| {
                if rng.gen::<bool>() {
                    first_genome[pos].clone()
                } else {
                    second_genome[pos].clone()
                }
            })
            .collect())
    }
}

impl<T: Clone> Recombinator<(Vec<T>, Vec<T>)> for UniformXo {
    type Output = Vec<T>;

    fn recombine(&self, genomes: (Vec<T>, Vec<T>), rng: &mut ThreadRng) -> Result<Self::Output> {
        self.recombine(<[Vec<T>; 2]>::from(genomes), rng)
    }
}

impl<G> Recombinator<[G; 2]> for UniformXo
where
    G: Crossover,
{
    type Output = G;

    fn recombine(
        &self,
        [mut first_genome, mut second_genome]: [G; 2],
        rng: &mut ThreadRng,
    ) -> Result<Self::Output> {
        ensure!(
            first_genome.size() == second_genome.size(),
            "Attempted to perform UniformXo on genomes of different length: {} and {}",
            first_genome.size(),
            second_genome.size()
        );
        let len = first_genome.size();
        for i in 0..len {
            if rng.gen::<bool>() {
                first_genome.crossover_gene(&mut second_genome, i)?;
            }
        }

        Ok(first_genome)
    }
}

impl<G> Recombinator<(G, G)> for UniformXo
where
    G: Crossover,
{
    type Output = G;

    fn recombine(&self, genomes: (G, G), rng: &mut ThreadRng) -> Result<Self::Output> {
        self.recombine(<[G; 2]>::from(genomes), rng)
    }
}
