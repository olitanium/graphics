use linear_algebra::Matrix;
pub use linear_algebra::Vector;

type Triangle<const DIMS: usize> = [Vector<DIMS>; 3];

/// https://inria.hal.science/inria-00072100/document
pub fn intersect3(one: Triangle<3>, two: Triangle<3>) -> bool {
    // TODO: handle degenerate triangles

    fn two_dimension_determinant(vec1: Vector<2>, vec2: Vector<2>, vec3: Vector<2>) -> f32 {
        let [xa, ya] = vec1.into_inner();
        let [xb, yb] = vec2.into_inner();
        let [xc, yc] = vec3.into_inner();
        Matrix::from_col_major([[xa - xc, xb - xc], [ya - yc, yb - yc]]).determinant()
    }

    fn three_dimension_determinant(
        veca: Vector<3>,
        vecb: Vector<3>,
        vecc: Vector<3>,
        vecd: Vector<3>,
    ) -> f32 {
        let [xa, ya, za] = veca.into_inner();
        let [xb, yb, zb] = vecb.into_inner();
        let [xc, yc, zc] = vecc.into_inner();
        let [xd, yd, zd] = vecd.into_inner();
        Matrix::from_col_major([
            [xa - xd, xb - xd, xc - xd],
            [ya - yd, yb - yd, yc - yd],
            [za - zd, zb - zd, zc - zd],
        ])
        .determinant()
    }

    fn make_determinants(one: Triangle<3>, two: Triangle<3>) -> [f32; 3] {
        let [p2, q2, r2] = two;
        one.map(|elem| three_dimension_determinant(p2, q2, r2, elem))
    }

    enum SignOrZero {
        Positive,
        Negative,
        Zero,
    }

    fn get_signorzero(float: f32) -> SignOrZero {
        if float > 0.0 {
            SignOrZero::Positive
        } else if float < 0.0 {
            SignOrZero::Negative
        } else {
            SignOrZero::Zero
        }
    }

    enum CrossType {
        None,
        Coplanar,
        Success(Triangle<3>, Triangle<3>),
    }

    macro_rules! permute {
        ($name:expr,0) => {{ $name }};
        ($name:expr,1) => {{
            let [a, b, c] = $name;
            [b, c, a]
        }};
        ($name:expr,2) => {{
            let [a, b, c] = $name;
            [c, a, b]
        }};
    }

    fn swap_qr(triangle: Triangle<3>) -> Triangle<3> {
        let [p, q, r] = triangle;

        [p, r, q]
    }

    fn determiant_crossing(one: Triangle<3>, two: Triangle<3>) -> CrossType {
        let determinants = make_determinants(one, two);
        use SignOrZero as S;
        match determinants.map(get_signorzero) {
            [S::Zero, S::Zero, S::Zero] => CrossType::Coplanar,
            [S::Positive, S::Positive, S::Positive] | [S::Negative, S::Negative, S::Negative] => {
                CrossType::None
            }
            // Only one vertex is in positive, this must be rotated around and the other triangle
            // kept as is
            [S::Positive, S::Negative | S::Zero, S::Negative | S::Zero] => {
                CrossType::Success(one, two)
            }
            [S::Negative | S::Zero, S::Positive, S::Negative | S::Zero] => {
                let permuted1 = permute!(one, 1);
                CrossType::Success(permuted1, two)
            }

            [S::Negative | S::Zero, S::Negative | S::Zero, S::Positive] => {
                let permuted1 = permute!(one, 2);
                CrossType::Success(permuted1, two)
            }
            // only one vertex is negative, this must be rotated around and the other triangle
            // permuted to make psositive.
            [S::Negative, S::Positive | S::Zero, S::Positive | S::Zero] => {
                let permuted2 = swap_qr(two);
                CrossType::Success(one, permuted2)
            }
            [S::Positive | S::Zero, S::Negative, S::Positive | S::Zero] => {
                let permuted1 = permute!(one, 1);
                let permuted2 = swap_qr(two);
                CrossType::Success(permuted1, permuted2)
            }
            [S::Positive | S::Zero, S::Positive | S::Zero, S::Negative] => {
                let permuted1 = permute!(one, 2);
                let permuted2 = swap_qr(two);
                CrossType::Success(permuted1, permuted2)
            }
            // One vertex is on the triangle, the other two on the same side. Make the others in the
            // negative and rename the zero-vertex to label 1
            [S::Zero, S::Negative, S::Negative] => CrossType::Success(one, two),
            [S::Negative, S::Zero, S::Negative] => {
                let permuted1 = permute!(one, 1);
                CrossType::Success(permuted1, two)
            }
            [S::Negative, S::Negative, S::Zero] => {
                let permuted1 = permute!(one, 2);
                CrossType::Success(permuted1, two)
            }
            [S::Zero, S::Positive, S::Positive] => {
                let permuted2 = swap_qr(two);
                CrossType::Success(one, permuted2)
            }
            [S::Positive, S::Zero, S::Positive] => {
                let permuted1 = permute!(one, 1);
                let permuted2 = swap_qr(two);
                CrossType::Success(permuted1, permuted2)
            }
            [S::Positive, S::Positive, S::Zero] => {
                let permuted1 = permute!(one, 2);
                let permuted2 = swap_qr(two);
                CrossType::Success(permuted1, permuted2)
            }
        }
    }

    fn smart_determinants_crossing(one: Triangle<3>, two: Triangle<3>) -> CrossType {
        match determiant_crossing(one, two) {
            nope @ (CrossType::None | CrossType::Coplanar) => nope,
            CrossType::Success(new_one, new_two) => match determiant_crossing(new_two, new_one) {
                nope @ (CrossType::None | CrossType::Coplanar) => nope,
                CrossType::Success(output1, output2) => CrossType::Success(output1, output2),
            },
        }
    }

    match smart_determinants_crossing(one, two) {
        CrossType::None => false,
        CrossType::Coplanar => {
            // TODO: colinear triangles
            todo!()
        }
        CrossType::Success(new_one, new_two) => {
            // TODO: intersecting planes and/or triangles
            use SignOrZero as S;
            assert!(matches!(
                make_determinants(one, two).map(get_signorzero),
                [S::Positive | S::Zero, S::Negative, S::Negative] | [S::Positive, S::Zero, S::Zero]
            ));

            assert!(matches!(
                make_determinants(two, one).map(get_signorzero),
                [S::Positive | S::Zero, S::Negative, S::Negative] | [S::Positive, S::Zero, S::Zero]
            ));

            let [p1, q1, r1] = new_one;
            let [p2, q2, r2] = new_two;

            three_dimension_determinant(p1, q1, p2, q2) <= 0.0
                && three_dimension_determinant(p1, r1, r2, p2) <= 0.0
        }
    }
}
