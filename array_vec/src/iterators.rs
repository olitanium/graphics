use crate::ArrayVec;

mod into_iter;
mod iter;

pub use into_iter::IntoIter;
pub use iter::Iter;

impl<T, const N: usize> FromIterator<T> for ArrayVec<T, N> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut out = Self::new();

        for elem in iter {
            match out.push_within_capacity(elem) {
                Ok(()) => {}
                Err(_) => return out,
            }
        }

        out
    }
}
