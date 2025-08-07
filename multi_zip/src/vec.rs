#[derive(Debug)]
pub struct HeapMultiZip<T>(Vec<T>);

impl<T> Iterator for HeapMultiZip<T>
where
    T: Iterator,
{
    type Item = Vec<T::Item>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.iter_mut().map(Iterator::next).collect()
    }
}

impl<F> HeapMultiZip<F> {
    #[inline]
    pub fn new<T>(iters: T) -> Self
    where
        T: IntoIterator,
        <T as IntoIterator>::Item: IntoIterator<IntoIter = F>,
    {
        Self(iters.into_iter().map(|x| x.into_iter()).collect())
    }
}

// T -> [S; N]
// T: IntoIterator<Item = S>
// S: IntoIterator<IntoIter = Vec::IntoIter>
