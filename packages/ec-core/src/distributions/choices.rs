use std::num::NonZeroUsize;

use rand::distributions::Slice;

/// A Distribution which knows how many choices are selcted from
pub trait ChoicesDistribution {
    /// The number of choices this distribution selects from
    fn num_choices(&self) -> NonZeroUsize;
}

impl<'a, T> ChoicesDistribution for Slice<'a, T> {
    fn num_choices(&self) -> NonZeroUsize {
        self.num_choices()
    }
}

impl<'a, T> ChoicesDistribution for &'a T
where
    T: ChoicesDistribution + ?Sized,
{
    fn num_choices(&self) -> NonZeroUsize {
        (**self).num_choices()
    }
}

impl<'a, T> ChoicesDistribution for &'a mut T
where
    T: ChoicesDistribution + ?Sized,
{
    fn num_choices(&self) -> NonZeroUsize {
        (**self).num_choices()
    }
}
