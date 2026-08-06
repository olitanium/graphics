use crate::Vector;

#[test]
fn reflect() {
    let vector = Vector::new([1.0, 1.0, -0.5]);
    let normal = Vector::new([0.0, 0.0, 1.0]).normalize();

    let reflected = vector.reflect(normal);

    assert_eq!(reflected, Vector::new([1.0, 1.0, 0.5]));
}

#[test]
fn flip() {
    assert_eq!(
        Vector::new([1.0, 1.0, 1.0]),
        Vector::new([-1.0, -1.0, -1.0]).flip()
    );
}

#[test]
fn scale() {
    assert_eq!(
        Vector::new([1.0, 1.0, 1.0]),
        Vector::new([0.5, 0.5, 0.5]).scale(2.0)
    );
}

#[test]
fn eq() {
    assert_eq!(Vector::new([1.0]), Vector::new([1.0]));

    assert_ne!(Vector::new([1.0]), Vector::new([0.0]));
}

#[test]
fn truncate() {
    let vec1 = Vector::new([1.0, 2.0, 3.0]);

    let vec2: Vector<4, f32> = vec1.truncate();
    let vec3: Vector<2, f32> = vec1.truncate();

    assert_eq!(vec2, Vector::new([1.0, 2.0, 3.0, 0.0]));
    assert_eq!(vec3, Vector::new([1.0, 2.0]));
}

#[test]
fn as_ref_test() {
    let x = [1.0, 2.0, 3.0_f32];

    let vector_x = Vector::new(x);

    assert_eq!(AsRef::<Vector<_>>::as_ref(&x), &vector_x);
}
