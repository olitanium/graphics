
pub use linear_algebra::Vector;

pub fn intersect(one: [Vector<3>; 3], two: [Vector<3>; 3]) -> bool {
    // TODO: handle degenerate triangles
    let [v_one_zeroth, v_one_first, v_one_second] = one;
    let [v_two_zeroth, v_two_first, v_two_second] = two;

    let one_normal = (v_one_first - v_one_zeroth).cross(v_one_second - v_one_zeroth);
    let two_normal = (v_two_first - v_two_zeroth).cross(v_two_second - v_two_zeroth);
    let d_two = two_normal.dot(v_two_zeroth);

    let one_signed_differences = one.map(|this_elem| two_normal.dot(this_elem) + d_two);

    // reject if all the signs are the same, and none are zero.
    const FLOAT_EPSILON: f32 = 0.001;
    let any_zero = one_signed_differences
        .into_iter()
        .any(|float| float.abs() < FLOAT_EPSILON);

    let same_sign = one_signed_differences
        .map(|float| float.signum())
        .array_windows()
        .all(|[a, b]| (a - b).abs() < FLOAT_EPSILON);

    if !any_zero && same_sign {
        return false;
    }

    let all_zero = Vector::new(one_signed_differences) == Vector::ZERO;
    if all_zero {
        // handle coplanar
        todo!()
    } else {
        // non-coplanar

        // L = o + td
        let _d = one_normal.cross(two_normal);
    }

    todo!()
}
