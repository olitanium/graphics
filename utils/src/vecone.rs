use core::num::NonZeroUsize;
use core::ops::{Deref, DerefMut};

pub type Iter<'a, T> = <&'a [T] as IntoIterator>::IntoIter;
pub type IterMut<'a, T> = <&'a mut [T] as IntoIterator>::IntoIter;
pub type IntoIter<T> = <Vec<T> as IntoIterator>::IntoIter;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct VecOne<T> {
    inner: Vec<T>,
}

impl<T> VecOne<T> {
    pub fn new(value: T) -> Self {
        Self { inner: vec![value] }
    }

    pub fn new_clone(value: T, count: NonZeroUsize) -> Self
    where
        T: Clone,
    {
        let inner = vec![value; count.get()];
        Self { inner }
    }

    pub fn push(&mut self, value: T) -> &mut T {
        self.inner.push_mut(value)
    }

    pub fn first(&self) -> &T {
        // SAFETY: guaranteed to have at least one element.
        unsafe { self.inner.first().unwrap_unchecked() }
    }

    pub fn first_mut(&mut self) -> &mut T {
        // SAFETY: guaranteed to have at least one element.
        unsafe { self.inner.first_mut().unwrap_unchecked() }
    }

    pub fn last(&self) -> &T {
        // SAFETY: guaranteed to have at least one element.
        unsafe { self.inner.last().unwrap_unchecked() }
    }

    pub fn last_mut(&mut self) -> &mut T {
        // SAFETY: guaranteed to have at least one element.
        unsafe { self.inner.last_mut().unwrap_unchecked() }
    }

    pub fn map<U, F>(self, func: F) -> VecOne<U>
    where
        F: FnMut(T) -> U,
    {
        let out = self.into_iter().map(func).collect();

        VecOne { inner: out }
    }

    pub fn try_from_iter<I>(iter: I) -> Option<Self>
    where
        I: IntoIterator<Item = T>,
    {
        let vec = Vec::from_iter(iter);
        Self::try_from_vec(vec)
    }

    pub fn try_from_vec(input: Vec<T>) -> Option<Self> {
        (!input.is_empty()).then_some(Self { inner: input })
    }

    pub fn iter(&self) -> Iter<'_, T> {
        self.inner.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        self.inner.iter_mut()
    }

    pub fn into_first_and_iter(self) -> (T, IntoIter<T>) {
        let mut iter = self.into_iter();

        // SAFETY: invariant that `VecOne::inner` has one element, so the first
        // can be popped off without issue.
        let first = unsafe { iter.next().unwrap_unchecked() };

        (first, iter)
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
        self.inner.extend(iter);
    }
}

impl<T> IntoIterator for VecOne<T> {
    type IntoIter = IntoIter<T>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a VecOne<T> {
    type IntoIter = Iter<'a, T>;
    type Item = &'a T;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut VecOne<T> {
    type IntoIter = IterMut<'a, T>;
    type Item = &'a mut T;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
