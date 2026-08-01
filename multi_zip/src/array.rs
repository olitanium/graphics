use array_vec::ArrayVec;

#[derive(Debug)]
pub struct MultiZip<T, const N: usize>([T; N]);

impl<T, const N: usize> Iterator for MultiZip<T, N>
where
    T: Iterator,
{
    type Item = [T::Item; N];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let out: Option<ArrayVec<T::Item, N>> = self.0.iter_mut().map(Iterator::next).collect();
        out.and_then(|inner| inner.try_into().ok())
    }
}

impl<T: Iterator, const N: usize> MultiZip<T, N> {
    pub fn new<I: IntoIterator<IntoIter = T>>(iters: [I; N]) -> Self {
        Self(iters.map(IntoIterator::into_iter))
    }
}

mod test {
    #[test]
    fn diff_len() {
        use crate::MultiZip;
        let x = vec![1, 2, 3, 4];
        let y = vec![2, 3, 4_usize];

        let out = MultiZip::new([x, y]).collect::<Vec<_>>();

        assert_eq!(out, [[1, 2], [2, 3], [3, 4]])
    }
}
