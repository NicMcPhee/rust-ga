#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]

use std::borrow::Borrow;
use std::fmt::{Display, Debug};

use rand::{rngs::ThreadRng, Rng};

use crate::individual::{Individual, TestResults};
use crate::population::Population;

pub type Bitstring = Vec<bool>;

pub fn make_random(len: usize, rng: &mut ThreadRng) -> Bitstring {
    (0..len).map(|_| rng.gen_bool(0.5)).collect()
}

#[must_use]
pub fn count_ones(bits: &[bool]) -> Vec<i64> {
    bits.iter().map(|bit| { if *bit { 1 } else { 0 }}).collect()
}

#[must_use]
pub fn hiff(bits: &[bool]) -> Vec<i64> {
    let num_scores = 2*bits.len() - 1;
    let mut scores = Vec::with_capacity(num_scores);
    do_hiff(bits, &mut scores);
    scores
}

pub fn do_hiff(bits: &[bool], scores: &mut Vec<i64>) -> bool {
    let len = bits.len();
    if len < 2 {
        scores.push(len as i64);
        true
    } else {
        let half_len = len / 2;
        let left_all_same = do_hiff(&bits[..half_len], scores);
        let right_all_same = do_hiff(&bits[half_len..], scores);
        if left_all_same && right_all_same && bits[0] == bits[half_len] {
            scores.push(bits.len() as i64);
            true
        } else {
            scores.push(0);
            false
        }
    }
}

#[must_use]
pub fn fitness_vec_to_test_results(results: Vec<i64>) -> TestResults<i64> {
    let total_result = results.iter().sum();
    TestResults { total_result, results }
}

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
        // We're probably OK losing a little precision if necessary here,
        // but we might want to look into either the `num_traits` or
        // `conv` crate for this conversion.
        let length: f32 = self.len() as f32;
        self.mutate_with_rate(1.0 / length, rng)
    }
}

#[cfg(test)]
mod genetic_operator_tests {
    use std::iter::zip;

    use super::*;

    // This test is stochastic, so I'm going to ignore it most of the time.
    #[test]
    #[ignore]
    fn mutate_one_over_does_not_change_much() {
        let mut rng = rand::thread_rng();
        let num_bits = 100;
        let parent_bits = make_random(num_bits, &mut rng);
        let child_bits = parent_bits.mutate_one_over_length(&mut rng);

        let num_differences = zip(parent_bits, child_bits).filter(|(p, c)| *p != *c).count();
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
        let parent_bits = make_random(num_bits, &mut rng);
        let child_bits = parent_bits.mutate_one_over_length(&mut rng);

        let num_differences = zip(parent_bits, child_bits).filter(|(p, c)| *p != *c).count();
        println!("Num differences = {num_differences}");
        assert!(0 < num_differences, "We're expecting at least one difference");
        assert!(num_differences < num_bits / 10, "We're not expecting lots of differences, and got {num_differences}.");
    }    
}

impl<R> Individual<Bitstring, R> {
    pub fn new_bitstring<H>(bit_length: usize, run_tests: impl Fn(&H) -> R, rng: &mut ThreadRng) -> Self
    where
        Bitstring: Borrow<H>,
        H: ?Sized
    {
        Self::new(
                |rng| make_random(bit_length, rng), 
                run_tests,
                rng)
    }
}

// TODO: Maybe change R to implement `Display` and have `TestResults` have a
//   nice-ish display function.
impl<R: Debug> Display for Individual<Bitstring, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = String::new();
        for bit in &self.genome {
            if *bit {
                result.push('1');
            } else {
                result.push('0');
            }
        }
        write!(f, "[{}]\n{:?}", result, self.test_results)
    }
}

impl<'a, R: Send> Population<'a, Bitstring, R> {
    pub fn new_bitstring_population<H>(
        pop_size: usize, 
        bit_length: usize, 
        run_tests: impl Fn(&H) -> R + Send + Sync) 
    -> Self
    where
        Bitstring: Borrow<H>,
        H: ?Sized
    {
        Self::new(
            pop_size,
            |rng| make_random(bit_length, rng),
            run_tests
        )
    }
}
