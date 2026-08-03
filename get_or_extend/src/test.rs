use crate::GetAtOrExtend as _;

#[test]
fn get_at_index() {
    let mut test = vec![0, 1, 2, 3, 4, 5];
    let ref_five = test.get_at_or_extend(5, || 0);
    assert_eq!(ref_five, &5);

    let ref_seven = test.get_at_or_extend(7, || 6);
    assert_eq!(ref_seven, &6);
    *ref_seven = 7;

    assert_eq!(test, [0, 1, 2, 3, 4, 5, 6, 7]);
}
