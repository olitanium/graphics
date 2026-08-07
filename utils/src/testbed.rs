use core::ops::Add;
use core::ptr;

/// Generic const implementation of `add_assign` for any T, U where T: Add. No
/// other trait bounds required.
#[inline]
pub const fn add_assign<T, U>(lhs: &mut T, rhs: U)
where
    T: [const] Add<U, Output = T>,
{
    // SAFETY: This is the only time that the MaybeUninit is read.
    let lhs_value = unsafe { ptr::read(lhs) };

    let out = Add::add(lhs_value, rhs);

    // SAFETY: write the value to prevent drop
    unsafe {
        ptr::from_mut(lhs).write(out);
    }
}

#[cfg(test)]
#[test]
fn drop_check_array_manipulation() {
    use core::sync::atomic::{AtomicUsize, Ordering};

    static HAS_DROPPED: AtomicUsize = AtomicUsize::new(0);

    struct DropCheck(usize);

    impl Drop for DropCheck {
        fn drop(&mut self) {
            HAS_DROPPED.fetch_add(self.0, Ordering::Relaxed);
        }
    }

    impl Add for DropCheck {
        type Output = Self;

        fn add(self, rhs: Self) -> Self::Output {
            let out = Self(self.0 + rhs.0);
            // at the end, both lhs and rhs are dropped.
            out
        }
    }

    let mut lhs = DropCheck(2);
    let rhs = DropCheck(3);

    assert_eq!(HAS_DROPPED.load(Ordering::Relaxed), 0);

    add_assign(&mut lhs, rhs);

    assert_eq!(HAS_DROPPED.load(Ordering::Relaxed), 2 + 3);

    drop(lhs);

    assert_eq!(HAS_DROPPED.load(Ordering::Relaxed), 2 + 3 + 5);
}
