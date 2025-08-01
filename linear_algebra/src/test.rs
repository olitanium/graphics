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
