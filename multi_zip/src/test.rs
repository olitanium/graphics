#[test]
fn diff_len() {
    use crate::MultiZip;
    let x = vec![1, 2, 3, 4];
    let y = vec![2, 3, 4_usize];

    let out = MultiZip::new([x, y]).collect::<Vec<_>>();

    assert_eq!(out, [[1, 2], [2, 3], [3, 4]]);
}
