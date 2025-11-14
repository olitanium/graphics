#![feature(maybe_uninit_slice)]
#![feature(maybe_uninit_array_assume_init)]
#![feature(const_convert)]
#![feature(const_trait_impl)]
#![feature(maybe_uninit_uninit_array_transpose)]

use std::fmt::Debug;
use std::mem::{self, ManuallyDrop, MaybeUninit};
use std::ops::{Deref, DerefMut};
mod iterators;
mod test;

pub use ArrayVec as StackVec;
pub use iterators::{IntoIter, Iter, IterMut};

mod initmacro;

pub struct ArrayVec<T, const N: usize> {
    len: usize,
    array: ManuallyDrop<[MaybeUninit<T>; N]>,
}

#[derive(Debug)]
struct LengthError<const N: usize>;

impl<T, const N: usize> ArrayVec<T, N> {
    pub const fn new() -> Self {
        Self {
            len: 0,
            array: ManuallyDrop::new([const { MaybeUninit::zeroed() }; N]),
        }
    }

    pub fn capacity(&self) -> usize {
        N
    }

    pub const fn push_within_capacity(&mut self, elem: T) -> Result<(), T> {
        if self.len >= N {
            return Err(elem);
        }

        self.push_lossy(elem);

        Ok(())
    }

    pub const fn push_lossy(&mut self, elem: T) {
        self.array[self.len] = MaybeUninit::new(elem);
        self.len += 1;
    }

    pub const fn const_collect<const M: usize>(array: [T; M]) -> Self {
        assert!(M <= N);
        let mut this = Self::new();
        let mut array = MaybeUninit::transpose(MaybeUninit::new(array));
        let mut index = 0;
        loop {
            if index == M {
                break;
            }
            let value = mem::replace(&mut array[index], MaybeUninit::uninit());
            this.push_lossy(unsafe { value.assume_init() });
            index += 1;
        }
        this
    }

    pub fn push(&mut self, elem: T) {
        self.push_within_capacity(elem)
            .map_err(|_| LengthError::<N>)
            .unwrap();
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

    pub fn as_slice(&self) -> &[T] {
        unsafe { self.array[..self.len].assume_init_ref() }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { self.array[..self.len].assume_init_mut() }
    }

    fn to_raw_parts(mut self) -> (usize, [MaybeUninit<T>; N]) {
        let len = mem::replace(&mut self.len, 0);
        let uninit_array = ManuallyDrop::new([const { MaybeUninit::uninit() }; N]);
        let md_array = mem::replace(&mut self.array, uninit_array);
        let array = ManuallyDrop::into_inner(md_array);
        (len, array)
    }

    pub fn try_into_array(self) -> Result<[T; N], TryFromErr> {
        let (len, array) = self.to_raw_parts();

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
        while self.pop().is_some() {}
    }
}

impl<T: Clone, const N: usize> Clone for ArrayVec<T, N> {
    fn clone(&self) -> Self {
        self.iter().cloned().collect()
    }
}

impl<T: Debug, const N: usize> Debug for ArrayVec<T, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArrayVec")
            .field("max_len", &N)
            .field("len", &self.len())
            .field("values", &self.as_slice())
            .finish()
    }
}
