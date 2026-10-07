use core::ops::{Deref, DerefMut};

pub struct VecOne<T> {
    inner: Vec<T>,
}

impl<T> VecOne<T> {
    pub fn new(value: T) -> Self {
        Self { inner: vec![value] }
    }

    pub fn push(&mut self, value: T) -> &mut T {
        self.inner.push_mut(value)
    }

    pub fn try_from_iter<I>(iter: I) -> Option<Self>
    where
        I: IntoIterator<Item = T>,
    {
        let mut iter = iter.into_iter();
        let first = iter.next()?;

        let mut out = Self::new(first);
        out.extend(iter);

        Some(out)
    }
}

impl<T> Deref for VecOne<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for VecOne<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T> Extend<T> for VecOne<T> {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        let iter = iter.into_iter();

        self.inner.extend(iter);
    }
}
