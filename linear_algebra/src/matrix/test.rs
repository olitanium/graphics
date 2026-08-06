use super::*;

#[test]
fn transpose() {
    #[rustfmt::skip]
    let mut y = Matrix::from_row_major([
        [1.0, 1.0],
        [1.0, 1.0],
        [1.0, 1.0],
        [1.0, 2.0],
    ]);

    y[(2, 1)] = 3.0;

    #[expect(
        clippy::float_cmp,
        reason = "no floating point maths to change value from literal"
    )]
    {
        assert_eq!(y[(2, 1)], 3.0);
    }

    let larger = y.truncate::<5, 3>();

    assert_eq!(
        larger,
        Matrix::from_col_major([
            [1.0, 1.0, 1.0, 1.0, 0.0],
            [1.0, 1.0, 3.0, 2.0, 0.0],
            [0.0, 0.0, 0.0, 0.0, 0.0],
        ])
    );
}
