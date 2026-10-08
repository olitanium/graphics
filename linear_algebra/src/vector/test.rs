use core::ops::Add;
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::vector::array_maths;
use crate::{Vector, VectorPreferredFloat, unit_vector, vector};

#[test]
fn drop_check_array_manipulation() {
    static HAS_DROPPED: AtomicUsize = AtomicUsize::new(0);

    struct DropCheck;

    impl Drop for DropCheck {
        fn drop(&mut self) {
            HAS_DROPPED.fetch_add(1, Ordering::Relaxed);
        }
    }

    impl Add for DropCheck {
        type Output = Self;

        fn add(self, _: Self) -> Self::Output {
            self
        }
    }

    // On add, DropCheck only drops the second one. This means that after the
    // add has occurred there will be one drop, and after the output is
    // dropped there will be two drops.

    let first = [const { DropCheck }; 3];
    let second = [const { DropCheck }; 3];

    assert_eq!(HAS_DROPPED.load(Ordering::Relaxed), 0);

    let sum = array_maths::add(first, second);

    assert_eq!(HAS_DROPPED.load(Ordering::Relaxed), 3);

    drop(sum);

    assert_eq!(HAS_DROPPED.load(Ordering::Relaxed), 6);
}

#[test]
fn reflect() {
    let vector = vector![1.0, 1.0, -0.5];
    let normal = vector![0.0, 0.0, 1.0].try_normalize().unwrap();

    let reflected = vector.reflect(normal);

    assert_eq!(reflected, vector![1.0, 1.0, 0.5]);
}

#[test]
fn flip() {
    assert_eq!(vector![1.0, 1.0, 1.0], -vector![-1.0, -1.0, -1.0]);
}

#[test]
fn scale() {
    assert_eq!(vector![1.0, 1.0, 1.0], vector![0.5, 0.5, 0.5].scale(2.0));
}

#[test]
fn eq() {
    assert_eq!(vector![1.0], vector![1.0]);

    assert_ne!(vector![1.0], vector![0.0]);
}

#[test]
fn truncate() {
    let vec1 = vector![1.0, 2.0, 3.0];

    let vec2: Vector<4, VectorPreferredFloat> = vec1.truncate();
    let vec3: Vector<2, VectorPreferredFloat> = vec1.truncate();

    assert_eq!(vec2, vector![1.0, 2.0, 3.0, 0.0]);
    assert_eq!(vec3, vector![1.0, 2.0]);
}

#[test]
fn as_ref_test() {
    let x = [1.0, 2.0, 3.0];

    let vector_x = Vector::new(x);

    assert_eq!(AsRef::<Vector<_>>::as_ref(&x), &vector_x);
}

#[test]
fn unit() {
    // let bad = unit_vector![0.0];
    let _x = unit_vector![1.0, 0.0];
}

#[test]
fn colinear() {
    vector![1.0, 1.0, 1.0_f32].colinear(vector![-0.5, -0.5, -0.5]);
}
