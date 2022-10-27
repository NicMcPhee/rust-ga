#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]

use clap::Parser;
use rust_ga::args::Args;
use rust_ga::do_main;

fn main() {
    let args = Args::parse();

    do_main(args);
}

#[cfg(test)]
mod tests {
    use rust_ga::bitstring::count_ones;
    use rust_ga::bitstring::hiff;
    use rust_ga::individual::Individual;
    use rust_ga::population::Population;
    // use rand::rngs::StdRng;
    // use rand::SeedableRng;
    // use std::time::Instant;
    // use test::Bencher;

    #[test]
    fn test_count_ones() {
        let bits = vec![true, false, true, false, true, false, true, false];
        assert_eq!(count_ones(&bits), vec![1, 0, 1, 0, 1, 0, 1, 0]);
    }

    #[test]
    fn test_hiff() {
        let bits = vec![true, false, false, false, true, true, true, true];
        assert_eq!(
            hiff(&bits),
            vec![1, 1, 0, 1, 1, 2, 0, 1, 1, 2, 1, 1, 2, 4, 0]
        );
    }

    #[test]
    fn test_individual_new() {
        let mut rng = rand::thread_rng();
        let ind = Individual::new_bitstring(128, count_ones, &mut rng);
        assert_eq!(ind.genome.len(), 128);
        assert_eq!(ind.scores, count_ones(&ind.genome));
        assert_eq!(ind.total_score, count_ones(&ind.genome).iter().sum());
    }

    #[test]
    fn test_population_new_count_ones() {
        let pop = Population::new_bitstring_population(100, 128, count_ones);
        assert_eq!(pop.individuals.len(), 100);
        assert_eq!(pop.individuals[0].genome.len(), 128);
        assert_eq!(
            pop.individuals[0].scores,
            count_ones(&pop.individuals[0].genome)
        );
        assert_eq!(
            pop.individuals[0].total_score,
            count_ones(&pop.individuals[0].genome).iter().sum()
        );
    }

    #[test]
    fn test_population_new_hiff() {
        let pop = Population::new_bitstring_population(100, 128, hiff);
        assert_eq!(pop.individuals.len(), 100);
        assert_eq!(pop.individuals[0].genome.len(), 128);
        assert_eq!(pop.individuals[0].scores, hiff(&pop.individuals[0].genome));
        assert_eq!(
            pop.individuals[0].total_score,
            hiff(&pop.individuals[0].genome).iter().sum()
        );
    }
}
