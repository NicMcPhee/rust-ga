#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]

use std::borrow::Borrow;

use rand::{rngs::ThreadRng, Rng};

use crate::individual::Individual;
use crate::population::Population;

pub type Bitstring = Vec<bool>;

pub trait LinearCrossover {
    #[must_use]
    fn uniform_xo(&self, other: &Self, rng: &mut ThreadRng) -> Self;
    #[must_use]
    fn two_point_xo(&self, other: &Self, rng: &mut ThreadRng) -> Self;
}

impl<T: Copy> LinearCrossover for Vec<T> {
    fn uniform_xo(&self, other: &Self, rng: &mut ThreadRng) -> Self {
        // The two parents should have the same length.
        assert!(self.len() == other.len());
        let len = self.len();
        (0..len).map(|i| 
            if rng.gen_bool(0.5) { 
                self[i] 
            } else { 
                other[i] 
            }).collect()
    }

    fn two_point_xo(&self, other: &Self, rng: &mut ThreadRng) -> Self {
        let len = self.len();
        // The two parents should have the same length.
        assert!(len == other.len());
        let mut genome = self.clone();
        let mut first = rng.gen_range(0..len);
        let mut second = rng.gen_range(0..len);
        if second < first {
            (first, second) = (second, first);
        }
        // We now know that first <= second
        genome[first..second].clone_from_slice(&other[first..second]);
        genome
    }
}

pub trait LinearMutation {
    #[must_use]
    fn mutate_with_rate(&self, mutation_rate: f32, rng: &mut ThreadRng) -> Self;
    #[must_use]
    fn mutate_one_over_length(&self, rng: &mut ThreadRng) -> Self;
}

impl LinearMutation for Bitstring {
    fn mutate_with_rate(&self, mutation_rate: f32, rng: &mut ThreadRng) -> Self {
        self.iter().map(|bit| {
            let r: f32 = rng.gen();
            if r < mutation_rate {
                !*bit
            } else {
                *bit
            }
        }).collect()
    }

    fn mutate_one_over_length(&self, rng: &mut ThreadRng) -> Self {
        let length = self.len() as f32;
        self.mutate_with_rate(1.0 / length, rng)
    }
}

#[must_use]
pub fn count_ones(bits: &[bool]) -> i64 {
    bits.iter().filter(|&&bit| bit).count() as i64
}

fn all_same(bits: &[bool]) -> bool {
    bits.iter().all(|&bit| bit == bits[0])
}

#[must_use]
pub fn hiff(bits: &[bool]) -> i64 {
    if bits.len() < 2 {
        bits.len() as i64
    } else {
        let half_len = bits.len() / 2;
        let mut score = hiff(&bits[..half_len]) + hiff(&bits[half_len..]);
        if all_same(bits) {
            score += bits.len() as i64;
        }
        score
    }
}

pub fn make_random(len: usize, rng: &mut ThreadRng) -> Bitstring {
    (0..len).map(|_| rng.gen_bool(0.5)).collect()
}

impl Individual<Bitstring> {
    pub fn new_bitstring<R>(bit_length: usize, compute_score: impl Fn(&R) -> i64 + Send + Sync, rng: &mut ThreadRng) -> Self
    where
        Bitstring: Borrow<R>,
        R: ?Sized
    {
        Self::new(
                |rng| make_random(bit_length, rng), 
                compute_score,
                rng)
    }
}

// TODO: I need to deal with the fact that this computes the score multiple times
// if I chain things like mutation and crossover. This is related to the need to
// parameterize the recombination operators, and I'll probably need to have some
// kind of vector of recombination operatorss that act on the Bitstrings, and then
// computes the score once at the end.
// 
// An alternative would be to use the Lazy eval tools and say that the score of
// an individual is computed lazily. That would mean that "intermediate" Individuals
// wouldn't have their score calculated since it's never used. That's a fairly
// heavy weight solution, though, so it would probably be nice to not go down
// that road if we don't have to.
//
// I also wonder if there are places where implementing the `From` trait would
// make sense. In principle we should be able to switch back and forth between
// `Bitstring` and `Individual` pretty freely, but I don't know if we can
// parameterize that with the score function.  
//
// This has hiff cooked in and needs to be parameterized on the score calculator.
impl Individual<Bitstring> {
    #[must_use]
    pub fn uniform_xo(&self, other_parent: &Self, rng: &mut ThreadRng) -> Self {
        let genome = self.genome.uniform_xo(&other_parent.genome, rng);
        let score = hiff(&genome);
        Self { genome, score }
    }

    #[must_use]
    pub fn two_point_xo(&self, other_parent: &Self, compute_score: impl Fn(&[bool]) -> i64, rng: &mut ThreadRng) -> Self {
        let genome = self.genome.two_point_xo(&other_parent.genome, rng);
        let score = compute_score(&genome);
        Self { genome, score }
    }

    #[must_use]
    pub fn mutate_one_over_length(&self, compute_score: impl Fn(&[bool]) -> i64, rng: &mut ThreadRng) -> Self {
        let new_genome = self.genome.mutate_one_over_length(rng);
        let score = compute_score(&new_genome);
        Self { genome: new_genome, score }
    }

    #[must_use]
    pub fn mutate_with_rate(&self, mutation_rate: f32, compute_score: impl Fn(&[bool]) -> i64, rng: &mut ThreadRng) -> Self {
        let new_genome: Vec<bool> = self.genome.mutate_with_rate(mutation_rate, rng);
        let score = compute_score(&new_genome);
        Self { genome: new_genome, score }
    }
}

#[cfg(test)]
mod test {
    use std::iter::zip;

    use super::*;

    // This test is stochastic, so I'm going to ignore it most of the time.
    #[test]
    #[ignore]
    fn mutate_one_over_does_not_change_much() {
        let mut rng = rand::thread_rng();
        let num_bits = 100;
        let parent: Individual<Bitstring> = Individual::new_bitstring(num_bits, count_ones, &mut rng);
        let child = parent.mutate_one_over_length(count_ones, &mut rng);

        let num_differences = zip(parent.genome, child.genome).filter(|(p, c)| *p != *c).count();
        println!("Num differences = {num_differences}");
        assert!(0 < num_differences, "We're expecting at least one difference");
        assert!(num_differences < num_bits / 10, "We're not expecting lots of differences, and got {num_differences}.");
    }

    // This test is stochastic, so I'm going to ignore it most of the time.
    #[test]
    #[ignore]
    fn mutate_with_rate_does_not_change_much() {
        let mut rng = rand::thread_rng();
        let num_bits = 100;
        let parent: Individual<Bitstring> = Individual::new_bitstring(num_bits, count_ones, &mut rng);
        let child = parent.mutate_with_rate(0.05, count_ones, &mut rng);

        let num_differences = zip(parent.genome, child.genome).filter(|(p, c)| *p != *c).count();
        println!("Num differences = {num_differences}");
        assert!(0 < num_differences, "We're expecting at least one difference");
        assert!(num_differences < num_bits / 10, "We're not expecting lots of differences, and got {num_differences}.");
    }    
}

impl Population<Bitstring> {
    pub fn new_bitstring_population<R>(
        pop_size: usize, 
        bit_length: usize, 
        compute_score: impl Fn(&R) -> i64 + Send + Sync) 
    -> Self
    where
        Bitstring: Borrow<R>,
        R: ?Sized
    {
        Self::new(
            pop_size,
            |rng| make_random(bit_length, rng),
            compute_score
        )
    }
}
