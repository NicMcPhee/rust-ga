use std::{borrow::Borrow, marker::PhantomData, num::NonZeroUsize};

use rand::{distributions::Uniform, prelude::Distribution};

use crate::distributions::wrappers::slice_cloning::EmptySlice;

#[derive(Debug, PartialEq)]
pub struct OneOfCloning<T, U> {
    collection: T,
    range: Uniform<usize>,
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
    pub fn new(val: T) -> Result<Self, EmptySlice> {
        match val.borrow().len() {
            0 => Err(EmptySlice),
            len => Ok(Self {
                collection: val,
                range: Uniform::new(0, len),
                _p: PhantomData,
            }),
        }
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn num_choices(&self) -> NonZeroUsize {
        let choices = self.collection.borrow().len();

        debug_assert_ne!(
            choices, 0,
            "This should never happen since the new method checks for this"
        );
        // FIXME: Check the performance of this
        // // Safety: at construction time, it was ensured that the slice was
        // // non-empty, as such the len is > 0.
        // unsafe { NonZeroUsize::new_unchecked(self.collection.borrow().len()) }
        #[allow(clippy::unwrap_used)]
        NonZeroUsize::new(choices).unwrap()
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
