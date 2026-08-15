//! A trait for safely accessing an out of bounds index by increasing the
//! container to the desired size.

#[cfg(test)]
mod test;

/// A trait for safely accessing an out of bounds index by increasing the
/// container to the desired size.
pub trait GetAtOrExtend {
    /// The type stored inside the container.
    type Out;

    /// Get a particular index, or slowly fill out the container with `new`
    /// until `index` is reached. Returning a mutable reference to the element
    /// at `index`.
    fn get_at_or_extend<F>(&mut self, index: usize, new: F) -> &mut Self::Out
    where
        F: FnMut() -> Self::Out;
}

impl<T> GetAtOrExtend for Vec<T> {
    type Out = T;

    #[inline]
    fn get_at_or_extend<F>(&mut self, index: usize, new: F) -> &mut Self::Out
    where
        F: FnMut() -> Self::Out,
    {
        if self.len() <= index {
            // we need to extend
            // index + 1 > index >= len, so we don't worry about shrinking
            self.resize_with(index + 1, new);
        }
        // SAFETY: we just made the vector index + 1 len, so index is the last element
        unsafe { self.get_unchecked_mut(index) }
    }
}
