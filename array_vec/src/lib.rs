//! A fixed capacity, variable length array, efficient for small collections
//! which don't need to be allocated.

#![no_std]
#![feature(maybe_uninit_array_assume_init)]
#![feature(const_trait_impl)]
#![feature(maybe_uninit_uninit_array_transpose)]
#![feature(const_destruct)]
#![feature(const_index)]
#![feature(const_array)]
#![feature(const_option_ops)]
#![feature(const_try)]
#![feature(const_drop_in_place)]
#![feature(const_iter)]
#![feature(const_result_unwrap_unchecked)]

mod initmacro;
mod iterators;
mod pop_front;

#[cfg(test)]
mod test;
use core::fmt::{self, Debug, Formatter};
use core::marker::Destruct;
use core::mem::{self, MaybeUninit};
use core::ops::{Deref, DerefMut, Index, IndexMut};
use core::{iter, ptr};

pub use iterators::{IntoIter, Iter, IterMut};
pub use pop_front::ArrayVecPopFront;
use utils::error_boilerplate;

/// In-place (for example, on-stack) growable vector, with a maximum size `N`.
/// Deref's to `[T]`, where most of it's methods come from.
pub struct ArrayVec<T, const N: usize> {
    len: usize,
    array: [MaybeUninit<T>; N],
}

#[derive(Debug)]
#[non_exhaustive]
/// Error returned from the `ArrayVec::try_push` method.
pub enum TryPushError {
    /// The closure returned `None`.
    ReturnedNone,
    /// The `ArrayVec` has no spare capacity.
    NoSpace,
}
error_boilerplate!(TryPushError);

/// Error returned from `ArrayVec::push_with` indicating that there is no spare
/// capacity.
#[derive(Debug)]
pub struct PushWithError;
error_boilerplate!(PushWithError);

impl<T, const N: usize> ArrayVec<T, N> {
    /// Create a new empty `ArrayVec`.
    #[inline]
    pub const fn new() -> Self {
        Self {
            len: 0,
            array: [const { MaybeUninit::zeroed() }; N],
        }
    }

    /// Create a new, full, `ArrayVec`.
    #[inline]
    pub const fn from_array(array: [T; N]) -> Self {
        Self {
            len: N,
            array: MaybeUninit::new(array).transpose(),
        }
    }

    /// Test the length of the `ArrayVec`, and only call the closure if there is
    /// space to insert the element. The closure returns an `Option<T>`, and
    /// will not insert any elements if the result is `None`.
    pub const fn try_push_with<F>(&mut self, mut func: F) -> Result<&mut T, TryPushError>
    where
        Option<T>: [const] Destruct,
        F: [const] FnMut() -> Option<T> + [const] Destruct,
    {
        if let Some(dest) = self.array.get_mut(self.len) {
            let Some(out) = func() else {
                return Err(TryPushError::ReturnedNone);
            };
            *dest = MaybeUninit::new(out);
            self.len += 1;
            // SAFETY: just filled above
            unsafe { Ok(dest.assume_init_mut()) }
        } else {
            Err(TryPushError::NoSpace)
        }
    }

    /// Test the length of the `ArrayVec`, and only call the closure if there is
    /// space to insert the element.
    pub const fn push_with<F>(&mut self, mut func: F) -> Result<&mut T, PushWithError>
    where
        F: [const] FnMut() -> T + [const] Destruct,
    {
        if let Some(dest) = self.array.get_mut(self.len) {
            *dest = MaybeUninit::new(func());
            self.len += 1;
            // SAFETY: just filled above
            unsafe { Ok(dest.assume_init_mut()) }
        } else {
            Err(PushWithError)
        }
    }

    /// Push a value, returning a mutable reference if successful, or an errow
    /// with the original value if unsuccessful.
    #[inline]
    pub const fn push(&mut self, elem: T) -> Result<&mut T, T> {
        if let Some(dest) = self.array.get_mut(self.len) {
            *dest = MaybeUninit::new(elem);
            self.len += 1;
            // SAFETY: just filled above
            unsafe { Ok(dest.assume_init_mut()) }
        } else {
            Err(elem)
        }
    }

    /// Collect an array (of any size) into a new `ArrayVec`, dropping any
    /// overflowing elements if the input is too long.
    pub const fn const_collect<const M: usize>(array: [T; M]) -> Self
    where
        T: [const] Destruct,
    {
        let array = ArrayVecPopFront::new(array);
        let mut out = Self::new();

        array.collect_into_other(&mut out);
        out
    }

    /// Pop off and return the last element of the list - the most recent one
    /// pushed.
    pub const fn pop(&mut self) -> Option<T> {
        // for security, replaces with zeroes to obscure previous values
        // `wrapping_sub` means if `self.len == 0` then `get_mut(usize::MAX) ==
        // None`
        let output = mem::replace(
            self.array.get_mut(self.len.wrapping_sub(1))?,
            const { MaybeUninit::zeroed() },
        );
        self.len -= 1;

        // SAFETY: self.array[self.len - 1] is within the initialized, it is
        // swapped for uninit, and the len is decremented
        Some(unsafe { output.assume_init() })
    }

    /// Get contents as slice.
    pub const fn as_slice(&self) -> &[T] {
        // SAFETY: self.len always points one past the end of the valid array,
        // so it is safe to both `get_unchecked` and `assume_init_ref`
        let slice = unsafe { self.array.get_unchecked(..self.len) };
        // SAFETY: as above
        unsafe { slice.assume_init_ref() }
    }

    /// Get contents as mutable slice.
    pub const fn as_mut_slice(&mut self) -> &mut [T] {
        // SAFETY: self.len always points one past the end of the valid array,
        // so it is safe to both `get_unchecked` and `assume_init_ref`
        let slice = unsafe { self.array.get_unchecked_mut(..self.len) };
        // SAFETY: as above
        unsafe { slice.assume_init_mut() }
    }

    const fn into_raw_parts(mut self) -> (usize, [MaybeUninit<T>; N]) {
        let len = mem::replace(&mut self.len, 0);
        let array = mem::replace(&mut self.array, [const { MaybeUninit::uninit() }; N]);
        #[expect(
            clippy::mem_forget,
            reason = "each component has been extracted, so the array does not need to be \
                      dropped. Adding this forget means we don't need `Self: [const] Destruct`"
        )]
        mem::forget(self);
        (len, array)
    }

    /// Try to turn an `ArrayVec` into a regular array. This will fail (and the
    /// original `ArrayVec` returned) if there is a size mismatch.
    #[inline]
    pub const fn try_into_array(self) -> Result<[T; N], TryFromErr<T, N>> {
        let (len, array) = self.into_raw_parts();

        if len == N {
            // SAFETY:
            // if len == N, the whole array is init
            Ok(unsafe { MaybeUninit::array_assume_init(array) })
        } else {
            Err(TryFromErr {
                array: Self { len, array },
            })
        }
    }

    /// Turn an `ArrayVec` into an array, using a closure to fill in the gaps.
    #[inline]
    pub const fn into_array_with<F>(mut self, mut gaps: F) -> [T; N]
    where
        F: [const] FnMut() -> T + [const] Destruct,
    {
        while self.push_with(&mut gaps).is_ok() {}

        // SAFETY: while loop above will only terminate when full.
        unsafe { self.try_into_array().unwrap_unchecked() }
    }

    #[inline]
    pub fn into_array_with_new_len<const M: usize, F>(self, gaps: F) -> [T; M]
    where
        F: FnMut() -> T,
    {
        let out: ArrayVec<_, _> = self.into_iter().chain(iter::repeat_with(gaps)).collect();
        // SAFETY: iter above has unlimited length, so the array_vec will always
        // be full.
        unsafe { out.try_into_array().unwrap_unchecked() }
    }

    #[inline]
    pub fn map<U, F>(self, f: F) -> ArrayVec<U, N>
    where
        F: FnMut(T) -> U,
    {
        self.into_iter().map(f).collect()
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> <&Self as IntoIterator>::IntoIter {
        self.into_iter()
    }

    pub fn iter_mut(&mut self) -> <&mut Self as IntoIterator>::IntoIter {
        self.into_iter()
    }
}

impl<T, const N: usize> Deref for ArrayVec<T, N> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T, const N: usize> DerefMut for ArrayVec<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<T, const N: usize> Default for ArrayVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct TryFromErr<T, const N: usize> {
    pub array: ArrayVec<T, N>,
}

impl<T, const N: usize> TryFrom<ArrayVec<T, N>> for [T; N] {
    type Error = TryFromErr<T, N>;

    fn try_from(value: ArrayVec<T, N>) -> Result<Self, Self::Error> {
        value.try_into_array()
    }
}

impl<T, const N: usize, const M: usize> TryFrom<[T; M]> for ArrayVec<T, N> {
    type Error = ();

    fn try_from(value: [T; M]) -> Result<Self, Self::Error> {
        if M <= N {
            let output = value.into_iter().collect();
            Ok(output)
        } else {
            Err(())
        }
    }
}

const impl<T, const N: usize> Drop for ArrayVec<T, N>
where
    T: [const] Destruct,
{
    fn drop(&mut self) {
        // SAFETY:
        // getting the mut_slice guarantees taking all init parts.
        // and like Vec<T>, the slice is dropped in place.
        unsafe {
            ptr::drop_in_place(self.as_mut_slice());
        }
        // safe alternative:
        // while self.pop().is_some() {}
    }
}

impl<T: Clone, const N: usize> Clone for ArrayVec<T, N> {
    fn clone(&self) -> Self {
        self.iter().cloned().collect()
    }
}

impl<T: Debug, const N: usize> Debug for ArrayVec<T, N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("ArrayVec")
            .field("capacity", &N)
            .field("len", &self.len())
            .field("values", &self.as_slice())
            .finish()
    }
}

impl<T, const N: usize, I> Index<I> for ArrayVec<T, N>
where
    [T]: Index<I>,
{
    type Output = <[T] as Index<I>>::Output;

    fn index(&self, index: I) -> &Self::Output {
        self.as_slice().index(index)
    }
}

impl<T, const N: usize, I> IndexMut<I> for ArrayVec<T, N>
where
    [T]: IndexMut<I>,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        self.as_mut_slice().index_mut(index)
    }
}
