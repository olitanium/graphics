use crate::ArrayVec;

pub type IterMut<'a, T> = <&'a mut [T] as IntoIterator>::IntoIter;

fn get_iter<T, const N: usize>(vec: &mut ArrayVec<T, N>) -> IterMut<'_, T> {
    vec.as_mut_slice().into_iter()
}

impl<'a, T, const N: usize> IntoIterator for &'a mut ArrayVec<T, N> {
    type IntoIter = IterMut<'a, T>;
    type Item = &'a mut T;

    fn into_iter(self) -> Self::IntoIter {
        get_iter(self)
    }
}