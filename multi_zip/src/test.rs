#[test]
fn diff_len() {
    use crate::MultiZip;
    let x = [1, 2, 3, 4].as_slice();
    let y = [2, 3, 4_usize].as_slice();

    let mut out = MultiZip::new([x, y]);

    assert_eq!(out.next(), Some([&1, &2]));
    assert_eq!(out.next(), Some([&2, &3]));
    assert_eq!(out.next(), Some([&3, &4]));
    assert_eq!(out.next(), None);
}
