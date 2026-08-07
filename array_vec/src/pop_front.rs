use core::marker::Destruct;
use core::mem::MaybeUninit;
use core::{mem, ptr};

use crate::ArrayVec;

pub struct ArrayVecPopFront<T, const N: usize> {
    value: [MaybeUninit<T>; N],
    front: usize,
}

impl<T, const N: usize> ArrayVecPopFront<T, N> {
    #[must_use]
    #[inline]
    pub const fn new(val: [T; N]) -> Self
    where
        T: [const] Destruct,
    {
        let value = val.map(MaybeUninit::new);
        Self { value, front: 0 }
    }

    #[must_use]
    #[inline]
    pub const fn as_slice(&self) -> &[T] {
        // SAFETY: `front` gives guaranteed front of the array.
        unsafe { self.value[self.front..].assume_init_ref() }
    }

    #[must_use]
    #[inline]
    pub const fn as_mut_slice(&mut self) -> &mut [T] {
        let slice = self.value.get_mut(self.front..).unwrap_or_default();
        // SAFETY: `front` gives guaranteed front of the array. If front overhangs,
        // returns an empty slice
        unsafe { slice.assume_init_mut() }
    }

    #[inline]
    #[must_use]
    pub const fn pop_front(&mut self) -> Option<T> {
        let front = self.value.get_mut(self.front)?;
        let init = mem::replace(front, MaybeUninit::zeroed());
        self.front += 1;
        // SAFETY: `self.front` points elsewhere, and cannot be used to ever read from
        // this zeroed memory again
        Some(unsafe { init.assume_init() })
    }

    pub const fn collect_into_other<const M: usize>(mut self, other: &mut ArrayVec<T, M>)
    where
        T: [const] Destruct,
    {
        while let Some(value) = self.pop_front() {
            match other.push(value) {
                Ok(_) => continue,
                Err(_) => return,
            }
        }
    }
}

const impl<T, const N: usize> Drop for ArrayVecPopFront<T, N>
where
    T: [const] Destruct,
{
    #[inline]
    fn drop(&mut self) {
        // SAFETY: values never read from again
        unsafe { ptr::drop_in_place(self.as_mut_slice()) }
    }
}

#[expect(clippy::inline_modules, reason = "clippy bug")]
#[cfg(test)]
mod tests {
    use core::sync::atomic::{AtomicUsize, Ordering};

    use crate::ArrayVecPopFront;

    #[test]
    fn drop_chk() {
        static DROPVAL: AtomicUsize = AtomicUsize::new(0);

        struct DropCheck(usize);
        impl Drop for DropCheck {
            fn drop(&mut self) {
                DROPVAL.fetch_add(self.0, Ordering::Relaxed);
            }
        }

        let array = [DropCheck(1), DropCheck(3), DropCheck(9)];
        let mut array_pop_front = ArrayVecPopFront::new(array);
        assert_eq!(DROPVAL.load(Ordering::Relaxed), 0);

        let opt_dropcheck = array_pop_front.pop_front();

        assert_eq!(DROPVAL.load(Ordering::Relaxed), 0);

        drop(opt_dropcheck);

        assert_eq!(DROPVAL.load(Ordering::Relaxed), 1);

        drop(array_pop_front);

        assert_eq!(DROPVAL.load(Ordering::Relaxed), 1 + 3 + 9);
    }
}
