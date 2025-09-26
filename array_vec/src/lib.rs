#![feature(maybe_uninit_slice)]
#![feature(maybe_uninit_array_assume_init)]

use std::mem::{self, ManuallyDrop, MaybeUninit};
use std::ops::{Deref, DerefMut};
mod iterators;
mod test;

pub use ArrayVec as StackVec;
pub use iterators::{IntoIter, Iter};

#[derive(Debug)]
pub struct ArrayVec<T, const N: usize> {
    len: usize,
    array: ManuallyDrop<[MaybeUninit<T>; N]>,
}

#[derive(Debug)]
struct LengthError<const N: usize>;

impl<T, const N: usize> ArrayVec<T, N> {
    pub fn new() -> Self {
        Self {
            len: 0,
            array: ManuallyDrop::new([const { MaybeUninit::uninit() }; N]),
        }
    }

    pub fn push_within_capacity(&mut self, elem: T) -> Result<(), T> {
        if self.len == N {
            return Err(elem);
        }

        self.array[self.len] = MaybeUninit::new(elem);
        self.len += 1;

        Ok(())
    }

    pub fn push(&mut self, elem: T) {
        self.push_within_capacity(elem)
            .map_err(|_| LengthError::<N>)
            .unwrap();
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }

        // security replaces with zeroes to obscure previous values
        let mut output = MaybeUninit::zeroed();
        mem::swap(&mut output, &mut self.array[self.len - 1]);
        self.len -= 1;

        Some(unsafe { output.assume_init() })
    }

    pub fn pop_unsecure(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }

        // security replaces with uninit
        let mut output = MaybeUninit::uninit();
        mem::swap(&mut output, &mut self.array[self.len - 1]);
        self.len -= 1;

        Some(unsafe { output.assume_init() })
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { (&self.array[..self.len]).assume_init_ref() }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { (&mut self.array[..self.len]).assume_init_mut() }
    }

    fn as_raw_parts(mut self) -> (usize, [MaybeUninit<T>; N]) {
        let len = self.len;
        let array = unsafe { ManuallyDrop::take(&mut self.array) };

        (len, array)
    }

    pub fn try_into_array(self) -> Result<[T; N], TryFromErr> {
        let (len, array) = self.as_raw_parts();

        if len != N {
            Err(TryFromErr {
                array_size: N,
                stack_vec_size: len,
            })
        } else {
            Ok(unsafe { MaybeUninit::array_assume_init(array) })
        }
    }

    pub fn map<U, F: FnMut(T) -> U>(self, f: F) -> ArrayVec<U, N> {
        self.into_iter().map(f).collect()
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

#[derive(Debug, Clone, Copy)]
#[expect(dead_code)]
pub struct TryFromErr {
    array_size: usize,
    stack_vec_size: usize,
}

impl<T, const N: usize> TryFrom<ArrayVec<T, N>> for [T; N] {
    type Error = TryFromErr;

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

impl<T, const N: usize> Drop for ArrayVec<T, N> {
    fn drop(&mut self) {
        while let Some(_) = self.pop_unsecure() {}
    }
}

impl<T: Clone, const N: usize> Clone for ArrayVec<T, N> {
    fn clone(&self) -> Self {
        self.iter().cloned().collect()
    }
}
