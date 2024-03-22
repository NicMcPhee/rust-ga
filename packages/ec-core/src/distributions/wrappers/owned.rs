use std::{borrow::Borrow, marker::PhantomData, num::NonZeroUsize};

use rand::{distributions::Uniform, prelude::Distribution};

use crate::distributions::{choices::ChoicesDistribution, wrappers::slice_cloning::EmptySlice};

#[derive(Debug, PartialEq, Eq)]
pub struct OneOfCloning<T, U> {
    collection: T,
    range: Uniform<usize>,
    num_choices: NonZeroUsize,
    _p: PhantomData<U>,
}

impl<T, U> OneOfCloning<T, U>
where
    T: Borrow<[U]>,
{
    /// Create a new [`OneOfCloning`] distribution, which selects a
    /// value from a collection and then returns a new value by cloning the
    /// selected value.
    ///
    /// # Errors
    /// - [`EmptySlice`] if an empty collection is passed in, since then no
    ///   element can be selected from there
    #[allow(clippy::unwrap_used)]
    #[allow(clippy::missing_panics_doc)]
    pub fn new(collection: T) -> Result<Self, EmptySlice> {
        let num_choices = NonZeroUsize::new(collection.borrow().len()).ok_or(EmptySlice)?;

        Ok(Self {
            collection,
            range: Uniform::new(0, num_choices.get()).unwrap(),
            num_choices,
            _p: PhantomData,
        })
    }
}
impl<T, U> ChoicesDistribution for OneOfCloning<T, U> {
    fn num_choices(&self) -> NonZeroUsize {
        self.num_choices
    }
}

impl<'a, T, U> Distribution<U> for OneOfCloning<T, U>
where
    T: Borrow<[U]>,
    U: 'a + Clone,
{
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> U {
        let idx = self.range.sample(rng);

        let slice = self.collection.borrow();

        debug_assert!(
            slice.len() >= idx,
            "This should never happen since the new method provides these guarantees"
        );

        // FIXME: Check the performance of this
        // // Safety: at construction time, it was ensured that the slice was
        // // non-empty, and that the `Uniform` range produces values in range
        // // for the slice
        // let val = unsafe { slice.get_unchecked(idx) }

        #[allow(clippy::unwrap_used)]
        let val = slice.get(idx).unwrap();

        val.clone()
    }
}
