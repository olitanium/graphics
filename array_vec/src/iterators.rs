use crate::ArrayVec;

mod into_iter;
mod iter;
mod iter_mut;

pub use into_iter::IntoIter;
pub use iter::Iter;
pub use iter_mut::IterMut;

impl<T, const N: usize> FromIterator<T> for ArrayVec<T, N> {
    #[inline]
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let mut out = Self::new();

        for elem in iter {
            match out.push(elem) {
                Ok(_) => {}
                Err(_) => return out,
            }
        }

        out
    }
}
