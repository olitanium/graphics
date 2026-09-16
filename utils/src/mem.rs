use core::ptr;

/// Map a value without dereferencing a mut ref.
///
/// This can be useful to implement methods such as `AddAssign` on non-clone
/// types, as the mutable reference can be read and written to in one go.
pub fn map<T, F>(input: &mut T, map: F)
where
    F: FnOnce(T) -> T,
{
    // SAFETY: This value is only read once, dropped in `map`, and written over
    // at the end.
    let input_copy = unsafe { ptr::read(input) };
    let new = map(input_copy);
    // SAFETY This value has already been dropped in `map`.
    unsafe {
        ptr::write(input, new);
    }
}

#[cfg(test)]
mod tests {
    use core::sync::atomic::AtomicUsize;
    use core::sync::atomic::Ordering::Relaxed;

    use crate::mem;

    #[test]
    fn test_drop() {
        static DROPCOUNT: AtomicUsize = AtomicUsize::new(0);

        struct Value(usize);

        impl Drop for Value {
            fn drop(&mut self) {
                DROPCOUNT.fetch_add(self.0, Relaxed);
            }
        }

        let mut value = Value(1);

        mem::map(&mut value, |value| {
            let inner = value.0;

            if inner == 2 { Value(1) } else { Value(2) }
        });

        assert_eq!(DROPCOUNT.load(Relaxed), 1);

        mem::map(&mut value, |value| {
            let inner = value.0;

            if inner == 2 { Value(1) } else { Value(2) }
        });

        assert_eq!(DROPCOUNT.load(Relaxed), 3);
    }
}
