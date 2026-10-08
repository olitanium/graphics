use linear_algebra::{Vector, vector};

use super::*;

#[test]
fn empty() {
    BezierSegment::<3, _>::new([]).get(0.5).unwrap_err();
}

#[test]
fn individual() {
    let points = [[0.0, 0.0], [1.0, 1.0]].map(Vector::new);

    let bezier = BezierSegment::new(points);

    for val in [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0] {
        let out = bezier.get(val).unwrap();
        let expected = vector![val, val];
        assert_eq!(out, expected, "at val {val}");
    }
}

#[test]
fn chain() {
    let start = vector![1.0, 0.0];
    let finish = vector![0.0, 1.0];

    let segment_one = BezierSegment::new([start, finish]);
    let segment_two = BezierSegment::new([finish, start]);

    let mut bezier = BezierCurve::new();
    bezier.push(segment_one).unwrap();
    bezier.push(segment_two).unwrap();

    assert_eq!(bezier.get(0.0).unwrap(), start);
    assert_eq!(bezier.get(0.5).unwrap(), finish);
    assert_eq!(bezier.get(1.0).unwrap(), start);

    assert_eq!(bezier.get_unnormalized(0.0).unwrap(), start);
    assert_eq!(bezier.get_unnormalized(1.0).unwrap(), finish);
    assert_eq!(bezier.get_unnormalized(2.0).unwrap(), start);
}

#[test]
#[expect(clippy::cast_possible_truncation, reason = "intended behaviour")]
#[expect(clippy::cast_sign_loss, reason = "intended behaviour")]
fn neg() {
    assert_eq!(-4.0_f32 as usize, 0);
}
