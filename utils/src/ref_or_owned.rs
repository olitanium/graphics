//! A type which could be owned, or could be a reference with lifetime `'a`.
//! This is useful as an owned type with no lifetimes when `'a` is `'static`.

use core::ops::Deref;

/// A type which could be owned, or could be a reference with lifetime `'a`.
/// This is useful as an owned type with no lifetimes when `'a` is `'static`.
#[derive(Debug, Clone)]
#[expect(
    clippy::exhaustive_enums,
    reason = "only two variants, ref or owned need to be defined"
)]
pub enum RefOwned<'a, T> {
    /// Owned type T.
    Owned(T),
    /// Reference type `&'a T`.
    Ref(&'a T),
}

impl<T> RefOwned<'_, T> {
    /// Get an owned value, by either destructuring the `Owned` or `Clone`ing
    /// the `Ref`.
    #[inline]
    pub fn get_owned(self) -> T
    where
        T: Clone,
    {
        match self {
            Self::Owned(value) => value,
            Self::Ref(reft) => reft.clone(),
        }
    }
}

impl<T> Deref for RefOwned<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        #[expect(
            clippy::match_same_arms,
            reason = "check commented code below, x has different lifetimes"
        )]
        match *self {
            Self::Owned(ref x) => x,
            Self::Ref(x) => x,
            // Self::Owned(ref x) | Self::Ref(x) => x,
        }
    }
}

impl<T> Default for RefOwned<'_, T>
where
    T: Default,
{
    #[inline]
    fn default() -> Self {
        Self::Owned(T::default())
    }
}
