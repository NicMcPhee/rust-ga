use anyhow::Context;
use rand::{rngs::ThreadRng, seq::SliceRandom};

pub mod collection;
/// Trait to represent types that can generate (possibly random) instances
/// of the generic type `T`. The implementing type provides whatever context
/// is necessary to generate instances of `T`, such as probabilities or
/// mechanisms for generating sub-types.
pub trait Generator<T> {
    /// # Errors
    ///
    /// This returns an `anyhow::Error` if the implementation of `generate`
    /// returns some sort of error. An example would be choosing a random
    /// item from a collection; this fails if the collection is empty.
    fn generate(&self, rng: &mut ThreadRng) -> anyhow::Result<T>;
}

/// Implement `Generator` for any reference to a `Generator`.
impl<'a, T, U> Generator<T> for &'a U
where
    U: Generator<T>,
{
    fn generate(&self, rng: &mut ThreadRng) -> anyhow::Result<T> {
        (**self).generate(rng)
    }
}

/// Implement `Generator` for any mutable reference to a `Generator`.
impl<'a, T, U> Generator<T> for &'a mut U
where
    U: Generator<T>,
{
    fn generate(&self, rng: &mut ThreadRng) -> anyhow::Result<T> {
        (**self).generate(rng)
    }
}

/// Generate a random element from an array of options.
///
/// # Errors
///
/// This returns an `anyhow::Error` if the array of options
/// is empty since we can't choose a random element from an
/// empty array.
impl<const N: usize, T> Generator<T> for [T; N]
where
    T: Clone,
{
    fn generate(&self, rng: &mut ThreadRng) -> anyhow::Result<T> {
        Ok(self
            .choose(rng)
            .context("`generate` called with an empty array of options to choose from")?
            .clone())
    }
}

// TODO: The implementation of `generate` here is essentially identical to the
// implementation   in `from_array`, which annoys me slightly. I thought about
// implementing `Generator` for   anything that can be turned into an iterator
// of `T`, but esitsu@Twitch pointed out that this   will be less efficient
// since we would have to sequentially process the elements of the iterator
//   instead of using random access. Since this is called _many_ times over a
// run, that seems bad.   I also thought have have a reference to a slice, but
// that would create lifetime issues in   the operators.

/// Generate a random element from a `Vec` of options.
///
/// # Errors
///
/// This returns an `anyhow::Error` if the array of options
/// is empty since we can't choose a random element from an
/// empty array.
impl<T> Generator<T> for Vec<T>
where
    T: Clone,
{
    fn generate(&self, rng: &mut ThreadRng) -> anyhow::Result<T> {
        Ok(self
            .choose(rng)
            .context("`generate` called with an empty collection of options to choose from")?
            .clone())
    }
}
