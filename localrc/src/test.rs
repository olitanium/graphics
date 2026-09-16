use core::sync::atomic::{AtomicBool, Ordering};
use std::hint::black_box;

use crate::LocalRc;

#[test]
fn drop() {
    struct HasDrop;

    static DROPCK: AtomicBool = AtomicBool::new(false);

    impl Drop for HasDrop {
        fn drop(&mut self) {
            assert!(!DROPCK.fetch_or(true, Ordering::Relaxed), "multiple drops");
        }
    }

    let many_has_drop = LocalRc::new(HasDrop);
    let many_has_drop2 = many_has_drop.clone();

    let many_has_drop3 = black_box(many_has_drop2.clone());
    let many_has_drop4 = black_box(many_has_drop.clone());

    black_box(many_has_drop3);
    black_box(many_has_drop4);
}

/// ``` no_compile
/// let mut value = LocalRc::new(1_usize);
///  *value = 2;
/// ```
#[test]
fn deref() {
    let value = LocalRc::new(1);

    assert_eq!(*value, 1);
}

/// # test
/// ``` no_compile
/// let value = LocalRc::new(Cell::new(1.0));
/// ```
#[test]
fn check_compile() {}
