/// Create a Distribution of a specified type, calling `.into()` on all
/// elements. Similar to the `[...]` syntax of rust optionally allowing to
/// specify the type using `uniform_distribution_of![<T> ...]`. To sample from
/// the Distribution, the elements are cloned. If you wish to avoid that, use
/// `uniform_distribution_of![ref <T> ...]` to return references instead.
///
/// This is equivalent to `arr_into![].into_distribution().unwrap()` without the
/// need for an `unwrap()`.
///
/// ![Railroad diagram for the `uniform_distribution_of!` macro][ref_text]
///
/// # Examples
/// ```rs
/// let distrb_1: impl Distribution<&i64> = uniform_distribution_of![ref <i64>
///     1i32,
///     2i32
/// ];
/// let distrb_2: impl Distribution<i64> = uniform_distribution_of![<i64>
///     1i32,
///     2i32
/// ];
/// ```
#[macro_railroad_annotation::generate_railroad("ref_text")]
#[macro_export]
macro_rules! uniform_distribution_of {
     (<$output_type:ty>  $($items:expr),+ $(,)?) => {
          // FIXME: is unwrap_unchecked a performance gain here?
          {
               #[allow(clippy::unwrap_used)]
               let val = ::std::result::Result::unwrap(
                    $crate::distributions::conversion
                         ::IntoDistribution::<$output_type>::into_distribution([
                         $(::std::convert::Into::<$output_type>::into($items)),+
                    ])
               );

               val
          }
     };
     ($($items:expr),+ $(,)?) => {
          // FIXME: is unwrap_unchecked a performance gain here?
          {
               #[allow(clippy::unwrap_used)]
               let val = ::std::result::Result::unwrap(
                    $crate::distributions::conversion::IntoDistribution::into_distribution([
                         $($items),+
                    ])
               );

               val
          }
     };
     (ref <$output_type:ty> $($items:expr),+ $(,)?) => {
          uniform_distribution_of![<$output_type>
               $(&$items)+
          ]
     };
     (ref  $($items:expr),+ $(,)?) => {
          uniform_distribution_of![
               $(&$items)+
          ]
     };
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod test {
    use crate::distributions::conversion::IntoDistribution;

    #[test]
    fn distr_into_owned() {
        let distr = uniform_distribution_of![<i64>
             1i32,
             3i32
        ];

        assert_eq!(distr, [1i64, 3i64].into_distribution().unwrap());
    }

    #[test]
    fn distr_inferred_owned() {
        let distr = uniform_distribution_of![1i32, 3i32];

        assert_eq!(distr, [1i32, 3i32].into_distribution().unwrap());
    }
}
