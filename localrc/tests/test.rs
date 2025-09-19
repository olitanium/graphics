use std::sync::Mutex;

use localrc::LocalRc;

#[test]
fn test_drop() {
    struct HasDrop;

    static DROPCK: Mutex<bool> = Mutex::new(false);

    impl Drop for HasDrop {
        fn drop(&mut self) {
            let mut lock = DROPCK.lock().unwrap();
            let has_changed = *lock;

            if has_changed {
                panic!("multiple drops")
            } else {
                *lock = true;
            }
        }
    }

    let many_has_drop = LocalRc::new(HasDrop);
    let many_has_drop2 = many_has_drop.clone();
    let _many_has_drop3 = many_has_drop2.clone();
    let _many_has_drop4 = many_has_drop.clone();
}

#[test]
fn test_deref() {
    let value = LocalRc::new(1.0);

    assert_eq!(*value, 1.0);

    // let mut value = LocalRc::new(1_usize);
    //*value = 2;
}

#[test]
fn check_compile() {
    // let value = LocalRc::new(Cell::new(1.0));
}
