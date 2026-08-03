use linear_algebra::Matrix;
pub use linear_algebra::Vector;

/// Triangle is just an array of Vectors.
type Triangle<const DIMS: usize> = [Vector<DIMS>; 3];

/// from [here](https://inria.hal.science/inria-00072100/document).
#[inline]
pub fn intersect3(one: Triangle<3>, two: Triangle<3>) -> bool {
    // TODO: handle degenerate triangles

    fn two_dimension_determinant(vec1: Vector<2>, vec2: Vector<2>, vec3: Vector<2>) -> f32 {
        let [xa, ya] = vec1.into_inner();
        let [xb, yb] = vec2.into_inner();
        let [xc, yc] = vec3.into_inner();
        Matrix::from_col_major([[xa - xc, xb - xc], [ya - yc, yb - yc]]).determinant()
    }

    #[inline]
    fn three_dimension_determinant(a: Vector<3>, b: Vector<3>, c: Vector<3>, d: Vector<3>) -> f32 {
        let [xa, ya, za] = a.into_inner();
        let [xb, yb, zb] = b.into_inner();
        let [xc, yc, zc] = c.into_inner();
        let [xd, yd, zd] = d.into_inner();
        Matrix::from_col_major([
            [xa - xd, xb - xd, xc - xd],
            [ya - yd, yb - yd, yc - yd],
            [za - zd, zb - zd, zc - zd],
        ])
        .determinant()
    }

    #[inline]
    fn make_determinants(one: Triangle<3>, two: Triangle<3>) -> [f32; 3] {
        let [p2, q2, r2] = two;
        one.map(|elem| three_dimension_determinant(p2, q2, r2, elem))
    }

    #[derive(Debug)]
    enum SignOrZero {
        Positive,
        Negative,
        Zero,
    }

    #[inline]
    fn get_signorzero(float: f32) -> SignOrZero {
        if float > 0.0 {
            SignOrZero::Positive
        } else if float < 0.0 {
            SignOrZero::Negative
        } else {
            SignOrZero::Zero
        }
    }

    fn make_determinants_signorzero(one: Triangle<3>, two: Triangle<3>) -> [SignOrZero; 3] {
        make_determinants(one, two).map(get_signorzero)
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

    #[inline]
    const fn swap_qr(triangle: Triangle<3>) -> Triangle<3> {
        let [p, q, r] = triangle;

        [p, r, q]
    }

    #[inline]
    #[expect(
        clippy::match_same_arms,
        reason = "better clarity on what each combination does"
    )]
    fn determiant_crossing(one: Triangle<3>, two: Triangle<3>) -> CrossType {
        use SignOrZero as S;
        match make_determinants_signorzero(one, two) {
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
            // flipped to make positive.
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

    let smart_determinants_crossing = {
        match determiant_crossing(one, two) {
            nope @ (CrossType::None | CrossType::Coplanar) => nope,
            CrossType::Success(new_one, new_two) => {
                make_determinants_signorzero(new_one, new_two);
                match determiant_crossing(new_two, new_one) {
                    nope @ (CrossType::None | CrossType::Coplanar) => nope,
                    CrossType::Success(output1, output2) => CrossType::Success(output1, output2),
                }
            }
        }
    };

    match smart_determinants_crossing {
        CrossType::None => false,
        CrossType::Coplanar => {
            // TODO: colinear triangles
            todo!()
        }
        CrossType::Success(new_one, new_two) => {
            // TODO: intersecting planes and/or triangles
            use SignOrZero as S;

            let test1 = make_determinants_signorzero(new_one, new_two);
            assert!(
                matches!(
                    test1,
                    [S::Positive | S::Zero, S::Negative, S::Negative]
                        | [S::Positive, S::Zero, S::Zero]
                ),
                "{test1:?}"
            );

            let test2 = make_determinants_signorzero(new_two, new_one);
            assert!(
                matches!(
                    test2,
                    [S::Positive | S::Zero, S::Negative, S::Negative]
                        | [S::Positive, S::Zero, S::Zero]
                ),
                "{test2:?}"
            );

            let [p1, q1, r1] = new_one;
            let [p2, q2, r2] = new_two;

            three_dimension_determinant(p1, q1, p2, q2) <= 0.0
                && three_dimension_determinant(p1, r1, r2, p2) <= 0.0
        }
    }
}
