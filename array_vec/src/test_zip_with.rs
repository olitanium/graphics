use core::array;
use core::marker::Destruct;

use crate::ArrayVecPopFront;

#[inline]
const fn zip_with<const N: usize, T, U, V, F>(array1: [T; N], array2: [U; N], mut func: F) -> [V; N]
where
    F: [const] FnMut(T, U) -> V + [const] Destruct,
    T: [const] Destruct,
    U: [const] Destruct,
    V: [const] Destruct,
{
    let mut array1 = ArrayVecPopFront::new(array1);
    let mut array2 = ArrayVecPopFront::new(array2);

    array::from_fn(const |_| {
        func(
            // SAFETY: equal sized arrays, will be empty exactly as the function stops calling.
            unsafe { array1.pop_front().unwrap_unchecked() },
            // SAFETY: as above.
            unsafe { array2.pop_front().unwrap_unchecked() },
        )
    })
}
