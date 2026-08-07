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

use core::fmt::Debug;
use core::marker::Destruct;
use core::mem::{self, MaybeUninit};
use core::ops::{Deref, DerefMut, Index, IndexMut};
use core::{iter, ptr};

mod iterators;

#[cfg(test)]
mod test;

pub use iterators::{IntoIter, Iter, IterMut};

mod initmacro;

mod pop_front;
pub use pop_front::ArrayVecPopFront;

pub struct ArrayVec<T, const N: usize> {
    len: usize,
    array: [MaybeUninit<T>; N],
}

#[derive(Debug)]
struct LengthError<const N: usize>;

impl<T, const N: usize> ArrayVec<T, N> {
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        Self {
            len: 0,
            array: [const { MaybeUninit::zeroed() }; N],
        }
    }

    #[inline]
    pub const fn from_array(array: [T; N]) -> Self {
        Self {
            len: N,
            array: MaybeUninit::new(array).transpose(),
        }
    }

    pub fn push_within_capacity_with<F>(&mut self, mut func: F) -> Result<&mut T, ()>
    where
        F: FnMut() -> T,
    {
        if self.len >= N {
            return Err(());
        }

        self.push(func()).map_err(|_| ())
    }

    /// Push a value, returning a mutable reference if successful, or the
    /// original value if unsuccessful
    pub const fn push(&mut self, elem: T) -> Result<&mut T, T>
where
        //T: [const] Destruct,
    {
        if let Some(dest) = self.array.get_mut(self.len) {
            *dest = MaybeUninit::new(elem);
            self.len += 1;
            // SAFETY: just filled above
            unsafe { Ok(dest.assume_init_mut()) }
        } else {
            Err(elem)
        }

        // self.array.get_mut(self.len).map_or_else(const || Err(elem), const
        // |dest| { dest = MaybeUninit::new(elem);
        // self.len += 1;
        // SAFETY: just filled above
        // unsafe { Ok(dest.assume_init_mut()) }
        // })
    }

    pub const fn const_collect<const M: usize>(array: [T; M]) -> Self
    where
        T: [const] Destruct,
    {
        // assert!(M <= N);
        // let mut this = Self::new();
        // let mut array = MaybeUninit::new(array).transpose();
        // let mut index = 0;
        // loop {
        // if index == M {
        // break;
        // }
        // let value = mem::replace(&mut array[index], MaybeUninit::uninit());
        // this.push_or_lose(unsafe { value.assume_init() });
        // index += 1;
        // }
        // this
        let array = ArrayVecPopFront::new(array);
        let mut out = Self::new();

        array.collect_into_other(&mut out);
        out
    }

    pub fn push_or_panic(&mut self, elem: T) {
        self.push(elem).map_err(|_| LengthError::<N>).unwrap();
    }

    pub const fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }

        // for security, replaces with zeroes to obscure previous values
        let output = mem::replace(
            &mut self.array[self.len - 1],
            const { MaybeUninit::zeroed() },
        );
        self.len -= 1;

        // SAFETY: self.array[self.len - 1] is within the initialized, it is swapped for
        // uninit, and the len is decremented
        Some(unsafe { output.assume_init() })
    }

    pub const fn as_slice(&self) -> &[T] {
        unsafe { self.array[..self.len].assume_init_ref() }
    }

    pub const fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { self.array[..self.len].assume_init_mut() }
    }

    fn into_raw_parts(mut self) -> (usize, [MaybeUninit<T>; N]) {
        let len = mem::replace(&mut self.len, 0);
        let uninit_array = [const { MaybeUninit::uninit() }; N];
        let array = mem::replace(&mut self.array, uninit_array);
        (len, array)
    }

    pub fn try_into_array(self) -> Result<[T; N], TryFromErr<T, N>> {
        let (len, array) = self.into_raw_parts();

        if len == N {
            Ok(unsafe { MaybeUninit::array_assume_init(array) })
        } else {
            Err(TryFromErr {
                array: ArrayVec { len, array },
            })
        }
    }

    pub fn into_array_with<F>(mut self, mut gaps: F) -> [T; N]
    where
        F: FnMut() -> T,
    {
        while self.push_within_capacity_with(&mut gaps).is_ok() {}

        // SAFETY: while loop above will only terminate when full.
        unsafe { self.try_into_array().unwrap_unchecked() }
    }

    pub fn into_array_with_new_len<const M: usize, F>(self, gaps: F) -> [T; M]
    where
        F: FnMut() -> T,
    {
        let out: ArrayVec<_, _> = self.into_iter().chain(iter::repeat_with(gaps)).collect();
        // SAFETY: iter above has unlimited length, so the array_vec will always be
        // full.
        unsafe { out.try_into_array().unwrap_unchecked() }
    }

    pub fn map<U, F>(self, f: F) -> ArrayVec<U, N>
    where
        F: FnMut(T) -> U,
    {
        self.into_iter().map(f).collect()
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
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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
