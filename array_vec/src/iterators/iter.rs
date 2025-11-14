use crate::ArrayVec;

pub type Iter<'a, T> = <&'a [T] as IntoIterator>::IntoIter;

fn get_iter<T, const N: usize>(vec: &ArrayVec<T, N>) -> Iter<'_, T> {
    vec.as_slice().iter()
}

impl<'a, T, const N: usize> IntoIterator for &'a ArrayVec<T, N> {
    type IntoIter = Iter<'a, T>;
    type Item = &'a T;

    fn into_iter(self) -> Self::IntoIter {
        get_iter(self)
    }
}
