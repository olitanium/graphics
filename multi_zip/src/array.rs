use array_vec::StackVec;

#[derive(Debug)]
pub struct MultiZip<T, const N: usize>([T; N]);

impl<T, const N: usize> Iterator for MultiZip<T, N>
where
    T: Iterator,
{
    type Item = [T::Item; N];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let out: Option<StackVec<T::Item, N>> = self.0.iter_mut().map(Iterator::next).collect();
        out.and_then(|inner| inner.try_into().ok())
    }
}

impl<T: Iterator, const N: usize> MultiZip<T, N> {
    pub fn new(iters: [T; N]) -> Self {
        Self(iters.map(IntoIterator::into_iter))
    }
}
