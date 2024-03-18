use std::fmt::Display;

use anyhow::bail;
use ec_core::{
    distributions::collection::{CollectionGenerator, ConvertToCollectionGenerator},
    genome::Genome,
};
use rand::{distributions::Standard, prelude::Distribution, rngs::ThreadRng, Rng};

use super::Linear;
use crate::recombinator::crossover::Crossover;

// TODO: Ought to have `LinearGenome<T>` so that `Bitstring` is just
//   `LinearGenome<bool>`.

pub struct BoolGenerator {
    pub true_probability: f64,
}

impl BoolGenerator {
    #[must_use]
    pub const fn new(true_probability: f64) -> Self {
        Self { true_probability }
    }
}

impl Distribution<bool> for BoolGenerator {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> bool {
        rng.gen_bool(self.true_probability)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Bitstring {
    pub bits: Vec<bool>,
}

impl<BG> Distribution<Bitstring> for CollectionGenerator<BG>
where
    BG: Distribution<bool>,
{
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Bitstring {
        Bitstring {
            bits: self.sample(rng),
        }
    }
}

impl Bitstring {
    pub fn random(num_bits: usize, rng: &mut ThreadRng) -> Self {
        Standard.into_collection_generator(num_bits).sample(rng)
    }

    pub fn random_with_probability(num_bits: usize, probability: f64, rng: &mut ThreadRng) -> Self {
        BoolGenerator::new(probability)
            .into_collection_generator(num_bits)
            .sample(rng)
    }

    pub fn iter(&self) -> std::slice::Iter<bool> {
        self.bits.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<bool> {
        self.bits.iter_mut()
    }
}

impl Display for Bitstring {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for bit in &self.bits {
            write!(f, "{}", u8::from(*bit))?;
        }
        Ok(())
    }
}

// TODO: Should we use copy-on-write when we clone genomes after selection?

impl<B> FromIterator<B> for Bitstring
where
    bool: From<B>,
{
    fn from_iter<T: IntoIterator<Item = B>>(iter: T) -> Self {
        Self {
            bits: iter.into_iter().map(From::from).collect(),
        }
    }
}

impl IntoIterator for Bitstring {
    type Item = bool;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.bits.into_iter()
    }
}

impl<'a> IntoIterator for &'a Bitstring {
    type Item = &'a bool;
    type IntoIter = std::slice::Iter<'a, bool>;

    fn into_iter(self) -> Self::IntoIter {
        self.bits.iter()
    }
}

impl<'a> IntoIterator for &'a mut Bitstring {
    type Item = &'a mut bool;
    type IntoIter = std::slice::IterMut<'a, bool>;

    fn into_iter(self) -> Self::IntoIter {
        self.bits.iter_mut()
    }
}

impl Genome for Bitstring {
    type Gene = bool;
}

impl Linear for Bitstring {
    fn size(&self) -> usize {
        self.bits.len()
    }

    fn gene_mut(&mut self, index: usize) -> Option<&mut Self::Gene> {
        self.bits.get_mut(index)
    }
}

impl Crossover for Bitstring {
    fn crossover_gene(&mut self, other: &mut Self, index: usize) -> anyhow::Result<()> {
        if let (Some(lhs), Some(rhs)) = (self.gene_mut(index), other.gene_mut(index)) {
            std::mem::swap(lhs, rhs);
            Ok(())
        } else {
            bail!("Crossing {self} and {other} at position {index} failed")
        }
    }

    fn crossover_segment(
        &mut self,
        other: &mut Self,
        range: std::ops::Range<usize>,
    ) -> anyhow::Result<()> {
        let lhs = &mut self.bits[range.clone()];
        let rhs = &mut other.bits[range.clone()];
        if lhs.len() == rhs.len() {
            lhs.swap_with_slice(rhs);
            Ok(())
        } else {
            bail!("Crossing {self} and {other} with range {range:?} failed")
        }
    }
}
