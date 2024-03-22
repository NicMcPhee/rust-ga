pub trait Population {
    type Individual;

    fn is_empty(&self) -> bool {
        self.size() == 0
    }

    fn size(&self) -> usize;
}

impl<I> Population for Vec<I> {
    type Individual = I;

    fn size(&self) -> usize {
        self.len()
    }
}

#[cfg(test)]
mod tests {
    use core::ops::Range;

    use rand::{prelude::Distribution, thread_rng, Rng};

    use crate::{distributions::collection::ConvertToCollectionGenerator, population::Population};

    struct RandValue {
        val: i32,
    }

    impl Distribution<RandValue> for Range<i32> {
        fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> RandValue {
            RandValue {
                val: rng.gen_range(self.clone()),
            }
        }
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn generator_works() {
        let mut rng = thread_rng();
        let population_size = 10;
        let range = -10..25;
        let vec_pop = range
            .to_collection_generator(population_size)
            .sample(&mut rng);

        assert_eq!(population_size, vec_pop.size());
        for i in vec_pop {
            assert!(range.contains(&i.val));
        }
    }
}
