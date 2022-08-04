use rand::{rngs::ThreadRng, Rng};
use rayon::prelude::{ParallelExtend, IntoParallelIterator, ParallelIterator};

use crate::individual::Individual;
use crate::population::Population;

pub type Bitstring = Vec<bool>;

pub fn count_ones(bits: &Bitstring) -> i64 {
    bits.iter().filter(|&&bit| bit).count() as i64
}

pub fn make_bitstring(len: usize, rng: &mut ThreadRng) -> Bitstring {
    (0..len).map(|_| rng.gen_bool(0.5)).collect()
}

impl Individual<Bitstring> {
    pub fn new_bitstring(bit_length: usize, compute_fitness: impl Fn(&Bitstring) -> i64, rng: &mut ThreadRng) -> Individual<Bitstring> {
        Individual::new(
                |rng| make_bitstring(bit_length, rng), 
                compute_fitness,
                rng)
    }
}

impl Population<Bitstring> {
    pub fn new(pop_size: usize, bit_length: usize) -> Population<Bitstring> {
        let mut pop = Vec::with_capacity(pop_size);
        // Using rayon's `par_extend` speeds up the population construction by a factor of 2
        // according to the Criterion benchmark results.
        pop.par_extend((0..pop_size)
            .into_par_iter()
            .map_init(
                rand::thread_rng,
                |rng, _| {
                    Individual::new_bitstring(bit_length, count_ones, rng)
                })
        );
        // let mut rng = rand::thread_rng();
        // for _ in 0..pop_size {
        //     let ind = Individual::new(bit_length, &mut rng);
        //     pop.push(ind);
        // }
        Population {
            individuals: pop,
        }
    }
}
