//! An `Rc` which does not reference the heap. Types stored in a `LocalRc` must
//! impl `Freeze`, a trait which ensures that there is no `UnsafeCell` and
//! therefore it is impossible for the seperate Rc to go out of sync. Once all
//! `LocalRc` have been dropped, the final value is also. There is still a write
//! to the heap on creation or drop.
//!
//!
//! `LocalRc` at present cannot be self-referential (there is no way to modify
//! the inside because of the `T: Freeze` bound) so there are no `Weak`
//! pointers.

#![feature(freeze)]

extern crate alloc;

use alloc::rc::Rc;
use core::borrow::Borrow;
use core::fmt::Debug;
use core::hash::Hash;
use core::marker::Freeze;
use core::mem::ManuallyDrop;
use core::ops::Deref;
use core::{cmp, fmt, hash, ptr};

#[cfg(test)]
mod test;

/// A locally stored reference counted smart pointer. See module docs for more.
pub struct LocalRc<T> {
    /// The data which is actually read.
    data: ManuallyDrop<T>,
    /// The real owner of the T, this is the only copy that gets dropped and the
    /// actual `Rc` is responsible for reference counting and `Drop`.
    /// `LocalRc` does not have a `Drop` impl because that is handled by the
    /// `Rc`.
    reference_count: Rc<T>,
}

impl<T: Freeze + Debug> Debug for LocalRc<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("LocalRc").field(&*self.data).finish()
    }
}

impl<T: Freeze + PartialEq> PartialEq for LocalRc<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.data.eq(&other.data)
    }
}

impl<T: Freeze + Eq> Eq for LocalRc<T> {}

impl<T: Freeze + Hash> Hash for LocalRc<T> {
    #[inline]
    fn hash<H>(&self, state: &mut H)
    where
        H: hash::Hasher,
    {
        self.data.hash(state);
    }
}

impl<T: Freeze + PartialOrd> PartialOrd for LocalRc<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.data.partial_cmp(&other.data)
    }
}

impl<T: Freeze + Default> Default for LocalRc<T> {
    #[inline]
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T: Freeze + Ord> Ord for LocalRc<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.data.cmp(&other.data)
    }
}

impl<T: Freeze> AsRef<T> for LocalRc<T> {
    #[inline]
    fn as_ref(&self) -> &T {
        &self.data
    }
}

impl<T: Freeze> Borrow<T> for LocalRc<T> {
    #[inline]
    fn borrow(&self) -> &T {
        &self.data
    }
}

impl<T: Freeze> LocalRc<T> {
    ///  Create a new `LocalRc` using the input value.
    #[inline]
    pub fn new(value: T) -> Self {
        // SAFETY:
        // the `ptr::read` is being placed into a ManuallyDrop to prevent dropping. It
        // will never be dropped when in this struct.
        // The `T: Freeze` bound ensures that the value in `data` cannot be modified
        // with a shared reference. this type does not allow the user to obtain
        // a mutable reference to `data`
        let data = unsafe { ManuallyDrop::new(ptr::read(&raw const value)) };
        let reference_count = Rc::new(value);
        Self {
            data,
            reference_count,
        }
    }

    /// akin to `Rc::strong_count` this reads from the heap to find the number
    /// of `LocalRc` still alive.
    #[must_use]
    #[inline]
    pub fn strong_count(this: &Self) -> usize {
        Rc::strong_count(&this.reference_count)
    }
}

impl<T: Freeze> Deref for LocalRc<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.data.deref()
    }
}

impl<T: Freeze> Clone for LocalRc<T> {
    #[inline]
    fn clone(&self) -> Self {
        // SAFETY:
        // the `ptr::read` is being placed into a ManuallyDrop to prevent dropping. It
        // will never be dropped when in this struct.
        // The `T: Freeze` bound ensures that the value in `data` cannot be modified
        // with a shared reference which ensures consistency with original.
        // this type does not allow the user to obtain
        // a mutable reference to `data`
        let data = unsafe { ManuallyDrop::new(ptr::read(&raw const *self.data)) };
        Self {
            data,
            reference_count: Rc::clone(&self.reference_count),
        }
    }
}
