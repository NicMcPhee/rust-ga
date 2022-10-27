#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]

use std::borrow::Borrow;

use rand::rngs::ThreadRng;

#[derive(Debug, Clone)]
pub struct Individual<T> {
    pub genome: T,
    // TODO: Maybe make the score here a new generic type S
    pub total_score: i64,
    pub scores: Vec<i64>,
}

impl<T> Individual<T> {
    /*
     * The type `R` is needed for circumstances where `T` is a "costly"
     * (to quote the documentation for the `Borrow` trait) type like
     * `Vec<bool>` when a "cheaper" type like `[bool]` would do. We might,
     * for example, prefer to have `compute_score` take a type like `&[bool]`,
     * but have everything written in terms of a more general (and "expensive")
     * type like `Vec<bool>`. If we use `Vec<bool>` for `T`, but specify
     * `compute_score` to take `&[bool]`, then the type checker won't be able
     * to link those things up.
     *
     * The use of `R` fixes that. Saying `T: Borrow<R>` says that `T` (e.g.,
     * `Vec<bool>`) can be borrowed as a reference to the simpler type (e.g.,
     * `[bool]`). So we can use `Vec<bool>` as our "general" type, but this
     * allows the system to know that it can convert (through borrowing) instances
     * of that to `[bool]`. Thus `compute_score` can now take `&[bool]` as an
     * argument and the types will work out.
     *
     * The `R: ?Sized` comes from the definition of the `Borrow` trait and is
     * necessary to say that `R` doesn't necessarily have a size that is known
     * at compile time. This is important because we're borrowing from `Vec<bool>`
     * (which has a known size) to `[bool]` (whose size depends on how many items
     * there are in the array, i.e., it's not known at compile time). Type generics
     * are assumed by default to be `Sized`, but we can make that optional with the
     * question mark `?Sized`.
     *
     * The idea for this came from @scottmcm's answer to a question on the
     * Rust user forum:
     * https://users.rust-lang.org/t/problem-passing-functions-as-arguments-and-deref/79491/2?u=nicmcphee
     * The documentation for the `Borrow` trait was very helpful:
     * https://doc.rust-lang.org/std/borrow/trait.Borrow.html
     */
    pub fn new<R>(
        make_genome: impl Fn(&mut ThreadRng) -> T,
        compute_score: impl Fn(&R) -> Vec<i64>,
        rng: &mut ThreadRng,
    ) -> Self
    where
        T: Borrow<R>,
        R: ?Sized,
    {
        let genome = make_genome(rng);
        let scores = compute_score(genome.borrow());
        Self {
            genome,
            total_score: scores.iter().sum(),
            scores,
        }
    }
}
