use super::*;

#[test]
fn invoke_and_print() {
    use super::matrix::Matrix;
    let _x = Matrix::from_row_major([
        [1.0, 1.0, 1.0, 2.0],
        [1.0, 1.0, 1.0, 2.0],
        [1.0, 1.0, 1.0, 2.0],
    ]);

    let mut y = Matrix::from_row_major([[1.0, 1.0], [1.0, 1.0], [1.0, 1.0], [1.0, 2.0]]);

    y[(2, 1)] = 3.0;

    assert!(y[(2, 1)] == 3.0);

    let larger = y.truncate::<5, 3>();

    println!("{:?}", larger);
}

#[test]
fn reflect() {
    use crate::Vector;
    let vector = Vector::new([1.0, 1.0, -0.5]);
    let normal = Vector::new([0.0, 0.0, 1.0]).normalize();

    let reflected = vector.reflect(normal);

    assert_eq!(reflected, Vector::new([1.0, 1.0, 0.5]));
}

#[test]
fn flip() {
    use crate::Vector;
    assert_eq!(
        Vector::new([1.0, 1.0, 1.0]),
        Vector::new([-1.0, -1.0, -1.0]).flip()
    )
}

#[test]
fn scale() {
    assert_eq!(
        Vector::new([1.0, 1.0, 1.0]),
        Vector::new([0.5, 0.5, 0.5]).scale(2.0)
    )
}

#[test]
fn eq() {
    use crate::Vector;
    assert_eq!(Vector::new([1.0]), Vector::new([1.0]));

    assert_ne!(Vector::new([1.0]), Vector::new([0.0]));
}

#[test]
fn truncate() {
    use crate::Vector;

    let vec1 = Vector::new([1.0, 2.0, 3.0]);

    let vec2: Vector<4, f32> = vec1.truncate();
    let vec3: Vector<2, f32> = vec1.truncate();

    assert_eq!(vec2, Vector::new([1.0, 2.0, 3.0, 0.0]));
    assert_eq!(vec3, Vector::new([1.0, 2.0]));
}
