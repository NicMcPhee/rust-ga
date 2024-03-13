/// This module contains the implementation of the `CollectionGenerator` struct
/// and related traits and functions.
///
/// The `CollectionGenerator` struct is used to generate a collection of random
/// elements. It takes an element generator and a size as input, and generates a
/// `Vec` of random elements based on the specified size and the mechanism for
/// generating random elements.
///
/// The module also defines the `ConvertToCollectionGenerator` trait, which
/// provides methods for converting a type into a `CollectionGenerator`. This
/// trait is implemented for any type that implements the `Generator` trait.
///
/// Finally, the module implements the `Generator` trait for the
/// `CollectionGenerator` struct, allowing it to generate a `Vec` of random
/// elements using the `generate` method.
use rand::prelude::Distribution;

/// Information for generating a collection of random elements.
///
/// `size` indicates how many elements to generate.
/// `element_generator` is used to generate individual elements.
#[allow(clippy::module_name_repetitions)]
pub struct CollectionGenerator<C> {
    pub element_generator: C,
    pub size: usize,
}

impl<C> CollectionGenerator<C> {
    /// Create a new `CollectionGenerator` with the given element generator and
    /// size.
    pub const fn new(element_generator: C, size: usize) -> Self {
        Self {
            element_generator,
            size,
        }
    }
}

/// Trait to convert a type (typically some sort of gene generator) into a
/// `CollectionGenerator` that generates collections of the specified size
/// of random elements (genes).
pub trait ConvertToCollectionGenerator {
    /// Convert the type into a `CollectionGenerator` that generates collections
    /// of the specified size, using `self` to generate the individual elements.
    /// This takes ownership of the type.
    fn into_collection_generator(self, size: usize) -> CollectionGenerator<Self>
    where
        Self: Sized;

    /// Convert a reference to the type into a `CollectionGenerator` that
    /// generates collections of the specified size, using `self` to
    /// generate the individual elements. This takes a reference to the type
    /// so the type can be used elsewhere when necessary.
    fn to_collection_generator(&self, size: usize) -> CollectionGenerator<&Self>;
}

impl<C> ConvertToCollectionGenerator for C
where
    C: ?Sized,
{
    /// Convert the type into a `CollectionGenerator` that generates collections
    /// of the specified size, using `self` to generate the individual elements.
    fn into_collection_generator(self, size: usize) -> CollectionGenerator<Self>
    where
        Self: Sized,
    {
        CollectionGenerator::new(self, size)
    }

    /// Convert a reference to the type into a `CollectionGenerator` that
    /// generates collections of the specified size, using `&self` to
    /// generate the individual elements.
    fn to_collection_generator(&self, size: usize) -> CollectionGenerator<&Self> {
        CollectionGenerator::new(self, size)
    }
}

/// Generate a `Vec` of random elements.
///
/// The number of element and the mechanism for generating
/// random elements are specified in the `CollectionGenerator`
/// struct.
///
/// # Errors
///
/// This returns an `anyhow::Error` generating any of
/// the elements returns an error.
impl<T, C> Distribution<Vec<T>> for CollectionGenerator<C>
where
    C: Distribution<T>,
{
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Vec<T> {
        (&self.element_generator)
            .sample_iter(rng)
            .take(self.size)
            .collect()
    }
}
