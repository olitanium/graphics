use crate::ArrayVec;

mod into_iter;
mod iter;
mod iter_mut;

pub use into_iter::IntoIter;
pub use iter::Iter;
pub use iter_mut::IterMut;

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
