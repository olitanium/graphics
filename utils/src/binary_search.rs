//! Search between two bounds, using a test function returning `cmp::Ordering`
//! and a function which calculates a mid-point between two values.

use core::cmp::Ordering;
use core::debug_assert_matches;

/// Binary search to find the `T` on the boundary between `test` returning
/// `Ordering::Greater` and `Ordering::Less`.
///
/// Returns None if both `test(&lower) == test(&upper)`. i.e. they are both too
/// low or too high.
///
/// This function may switch `upper` and `lower` if they are the wrong way
/// around, adding two more `test`s.
pub fn binary_search<T, F, O>(
    mut lower: T,
    mut upper: T,
    make_midpoint: F,
    mut test: O,
    count: usize,
) -> Option<T>
where
    F: Fn(&T, &T) -> T,
    O: FnMut(&T) -> Ordering,
{
    (lower, upper) = match (test(&lower), test(&upper)) {
        (Ordering::Less, Ordering::Greater) => (lower, upper),
        (Ordering::Equal, _) => return Some(lower),
        (_, Ordering::Equal) => return Some(upper),
        (Ordering::Greater, Ordering::Less) => (upper, lower),
        (Ordering::Less, Ordering::Less) => return None,
        (Ordering::Greater, Ordering::Greater) => return None,
    };

    Some(binary_search_unchecked(
        lower,
        upper,
        make_midpoint,
        test,
        count,
    ))
}

/// Binary search to find the `T` on the boundary between `test` returning
/// `Ordering::Greater` and `Ordering::Less`.
///
/// Undefined behavior if `test(&lower) != Ordering::Less` and
/// `test(&upper) != Ordering::Greater`.
pub fn binary_search_unchecked<T, F, O>(
    mut lower: T,
    mut upper: T,
    make_midpoint: F,
    mut test: O,
    count: usize,
) -> T
where
    F: Fn(&T, &T) -> T,
    O: FnMut(&T) -> Ordering,
{
    debug_assert_matches!(test(&lower), Ordering::Less);
    debug_assert_matches!(test(&upper), Ordering::Greater);

    // 1..count because one more make_midpoint occurs below
    for _ in 1..count {
        let midpoint = make_midpoint(&lower, &upper);

        let test_out = test(&midpoint);
        match test_out {
            Ordering::Greater => upper = midpoint,
            Ordering::Equal => return midpoint,
            Ordering::Less => lower = midpoint,
        }
    }

    make_midpoint(&lower, &upper)
}

#[cfg(test)]
mod tests {
    use crate::binary_search::binary_search;

    #[test]
    fn floating_point() {
        let x = binary_search(
            0.0_f32,
            1.0,
            |&lb, &ub| f32::midpoint(lb, ub),
            |&x| x.total_cmp(&0.3),
            4,
        );
        assert_eq!(x, Some(0.3125));
    }
}
