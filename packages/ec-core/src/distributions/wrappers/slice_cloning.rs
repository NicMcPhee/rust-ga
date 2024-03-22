use std::num::NonZeroUsize;

use rand::{distributions::Slice, prelude::Distribution};

use crate::distributions::choices::ChoicesDistribution;

/// Generate a random element from an array of options, cloning the choosen
/// element.
#[derive(Debug, Clone, Copy)]
pub struct SliceCloning<'a, T>(Slice<'a, T>);

#[derive(Debug, Clone, Copy)]
pub struct EmptySlice;

impl core::fmt::Display for EmptySlice {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Tried to create a `distributions::Slice` with an empty slice"
        )
    }
}

impl std::error::Error for EmptySlice {}

impl<'a, T> SliceCloning<'a, T> {
    /// Create a new `Slice` instance which samples uniformly from the slice.
    /// Returns `Err` if the slice is empty.
    ///
    /// # Errors
    /// - [`EmptySlice`] if the passed in slice is empty
    pub fn new(slice: &'a [T]) -> Result<Self, EmptySlice> {
        Ok(Self(Slice::new(slice).map_err(|_| EmptySlice)?))
    }
}

impl<'a, T> ChoicesDistribution for SliceCloning<'a, T> {
    fn num_choices(&self) -> NonZeroUsize {
        self.0.num_choices()
    }
}

impl<'a, T> Distribution<T> for SliceCloning<'a, T>
where
    T: Clone,
{
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> T {
        self.0.sample(rng).clone()
    }
}
