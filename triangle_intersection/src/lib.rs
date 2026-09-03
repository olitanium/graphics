//! Test triangles (as arrays of `linear_algebra::Vector<DIMS>`) for
//! intersection.
#![warn(missing_docs)]
#![expect(clippy::too_many_lines, reason = "I don't mind")]

use geometry::{Orientation, Pose, TransformBy as _};
pub use linear_algebra::Vector;
use linear_algebra::{Matrix, UnitVector};

/// Triangle is just an array of Vectors.
type Triangle<const DIMS: usize> = [Vector<DIMS>; 3];

/// Is the input Positive, Negative or Zero?
#[derive(Debug)]
enum SignOrZero {
    /// `>0.0`.
    Positive,
    /// `<0.0`.
    Negative,
    /// `0.0`.
    Zero,
}

impl SignOrZero {
    /// Create new from float.
    #[inline]
    fn new(float: f32) -> Self {
        if float > 0.0 {
            Self::Positive
        } else if float < 0.0 {
            Self::Negative
        } else {
            Self::Zero
        }
    }
}

/// Macro to panic-free premute a 3-length array.
/// index 0 moves to the given index.
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

/// from [here](https://inria.hal.science/inria-00072100/document).
#[inline]
#[expect(clippy::missing_panics_doc, reason = "will be fixed")]
#[must_use]
pub fn intersect3(one: Triangle<3>, two: Triangle<3>) -> bool {
    /// calculate a three dimensional determinant as features in [here](https://inria.hal.science/inria-00072100/document).
    #[inline]
    #[must_use]
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
    #[must_use]
    fn make_determinants(one: Triangle<3>, two: Triangle<3>) -> [f32; 3] {
        let [p2, q2, r2] = two;
        one.map(|elem| three_dimension_determinant(p2, q2, r2, elem))
    }

    #[must_use]
    #[inline]
    fn make_determinants_signorzero(one: Triangle<3>, two: Triangle<3>) -> [SignOrZero; 3] {
        make_determinants(one, two).map(SignOrZero::new)
    }

    enum CrossType {
        None,
        Coplanar,
        Success(Triangle<3>, Triangle<3>),
        MaybeIntersecting,
    }

    #[inline]
    #[must_use]
    const fn swap_qr(triangle: Triangle<3>) -> Triangle<3> {
        let [p, q, r] = triangle;

        [p, r, q]
    }

    #[inline]
    #[expect(
        clippy::match_same_arms,
        reason = "better clarity on what each combination does"
    )]
    #[must_use]
    fn determiant_crossing(one: Triangle<3>, two: Triangle<3>) -> CrossType {
        use SignOrZero as S;
        match make_determinants_signorzero(one, two) {
            [S::Zero, S::Zero, S::Zero] => CrossType::Coplanar,
            [S::Positive, S::Positive, S::Positive] => CrossType::MaybeIntersecting,
            [S::Negative, S::Negative, S::Negative] => CrossType::None,
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
            CrossType::Success(new_one, new_two) => match determiant_crossing(new_two, new_one) {
                nope @ (CrossType::None | CrossType::Coplanar | CrossType::MaybeIntersecting) => {
                    nope
                }
                CrossType::Success(output1, output2) => CrossType::Success(output1, output2),
            },
            CrossType::MaybeIntersecting => {
                match determiant_crossing(two, one) {
                    CrossType::MaybeIntersecting => CrossType::MaybeIntersecting,
                    CrossType::None => CrossType::None,
                    // I don't expect these to come up because we've already asserted that they are
                    // not coplanar or intersecting
                    CrossType::Coplanar | CrossType::Success(..) => CrossType::None,
                }
            }
        }
    };

    match smart_determinants_crossing {
        CrossType::None => false,
        CrossType::Coplanar => embed_three_to_two(one, two)
            .is_some_and(|(new_one_trunc, new_two_trunc)| intersect2(new_one_trunc, new_two_trunc)),
        CrossType::Success(new_one, new_two) => {
            // TODO: intersecting planes and/or triangles
            use SignOrZero as S;
            #[cfg(debug_assertions)]
            {
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
            }

            let [p1, q1, r1] = new_one;
            let [p2, q2, r2] = new_two;

            three_dimension_determinant(p1, q1, p2, q2) <= 0.0
                && three_dimension_determinant(p1, r1, r2, p2) <= 0.0
        }
        CrossType::MaybeIntersecting => {
            let one_way = || {
                embed_three_to_two(one, two).is_some_and(|(new_one_trunc, new_two_trunc)| {
                    intersect2(new_one_trunc, new_two_trunc)
                })
            };

            let two_way = || {
                embed_three_to_two(two, one).is_some_and(|(new_two_trunc, new_one_trunc)| {
                    intersect2(new_two_trunc, new_one_trunc)
                })
            };

            one_way() && two_way()
        }
    }
}

fn embed_three_to_two(one: Triangle<3>, two: Triangle<3>) -> Option<(Triangle<2>, Triangle<2>)> {
    let [a, b, c] = one;
    // if ab and ac are parallel, then the vector is just a line and is not
    // handled yet
    let normal = (b - a).cross(c - a).try_normalize()?;

    // Should not fail because `X` != `Y`
    let orientation = Orientation::new_forward_up_try_two(normal, UnitVector::Y, UnitVector::X)?;

    let transform = Pose::new_from_orientation_translation(orientation, a);
    let inverse_transform = transform.inverse();

    let new_one = one.transform_by(&inverse_transform);
    let new_two = two.transform_by(&inverse_transform);

    let new_one_trunc = new_one.map(Vector::truncate);
    let new_two_trunc = new_two.map(Vector::truncate);

    Some((new_one_trunc, new_two_trunc))
}

#[inline]
#[must_use]
/// 2D triangle intersection.
pub fn intersect2(one: Triangle<2>, two: Triangle<2>) -> bool {
    use SignOrZero as S;

    /// calculate a two dimensional determinant as features in [here](https://inria.hal.science/inria-00072100/document).
    fn twod_det(vec1: Vector<2>, vec2: Vector<2>, vec3: Vector<2>) -> SignOrZero {
        let [xa, ya] = vec1.into_inner();
        let [xb, yb] = vec2.into_inner();
        let [xc, yc] = vec3.into_inner();
        let det = Matrix::from_col_major([[xa - xc, xb - xc], [ya - yc, yb - yc]]).determinant();
        SignOrZero::new(det)
    }

    fn make_counterclockwise(tri: Triangle<2>) -> Triangle<2> {
        let [p, q, r] = tri;
        let orientation = twod_det(p, q, r);
        match orientation {
            S::Positive | S::Zero => tri,
            S::Negative => [p, r, q],
        }
    }

    let ccw_one = make_counterclockwise(one);

    let [p, ..] = ccw_one;
    let ccw_two = make_counterclockwise(two);

    fn test_vertex_wrt_sides(triangle: Triangle<2>, test: Vector<2>) -> [SignOrZero; 3] {
        let [p, q, r] = triangle;
        [(p, q), (q, r), (r, p)].map(|(start, finish)| twod_det(start, finish, test))
    }

    fn interior_r1(ccw_one: Triangle<2>, ccw_two: Triangle<2>, old_two: &[SignOrZero; 3]) -> bool {
        #[cfg(debug_assertions)]
        {
            let [p, ..] = ccw_one;
            let out = test_vertex_wrt_sides(ccw_two, p);

            debug_assert!(
                matches!(out, [S::Positive, S::Positive, S::Negative]),
                "didn't rotate correctly: {old_two:?} => {out:?}"
            );
        }

        let [p1, q1, r1] = ccw_one;
        let [p2, _, r2] = ccw_two;

        let a_branch = || {
            if matches!(twod_det(r2, p1, q1), S::Negative) {
                return false;
            }
            if matches!(twod_det(p1, p2, q1), S::Positive | S::Zero) {
                return true;
            }
            if matches!(twod_det(p1, p2, r1), S::Negative) {
                return false;
            }
            match twod_det(q1, r1, p2) {
                S::Positive | S::Zero => true,
                S::Negative => false,
            }
        };

        let b_branch = || {
            if matches!(twod_det(r2, p2, r1), S::Negative) {
                return false;
            }
            if matches!(twod_det(q1, r1, r2), S::Negative) {
                return false;
            }
            // Deviation from the source, this is the other wy
            match twod_det(p1, p2, r1) {
                S::Negative => false,
                S::Positive | S::Zero => true,
            }
        };

        match twod_det(r2, p2, q1) {
            S::Positive | S::Zero => a_branch(),
            S::Negative => b_branch(),
        }
    }

    #[expect(
        clippy::unnested_or_patterns,
        reason = "clarity from original material"
    )]
    #[expect(clippy::similar_names, reason = "as in original material")]
    fn interior_boundary_r2(
        ccw_one: Triangle<2>,
        ccw_two: Triangle<2>,
        old_two: &[SignOrZero; 3],
    ) -> bool {
        #[cfg(debug_assertions)]
        {
            let [p, ..] = ccw_one;

            let out = test_vertex_wrt_sides(ccw_two, p);

            debug_assert!(
                matches!(
                    out,
                    // interior of R2
                    [S::Positive, S::Negative, S::Negative]
                // boundary of R2
                | [S::Positive, S::Negative, S::Zero]
                | [S::Positive, S::Zero, S::Negative]
                ),
                "didn't rotate correctly: {old_two:?} => {out:?}"
            );
        }

        let [p1, q1, r1] = ccw_one;
        let [p2, q2, r2] = ccw_two;

        let step_va = || match twod_det(r2, p2, r1) {
            S::Negative => false,
            S::Positive | S::Zero => true,
        };

        let step_vb = || match twod_det(q1, r1, q2) {
            S::Negative => false,
            S::Positive | S::Zero => true,
        };

        let step_vc = || match twod_det(q2, r2, r1) {
            S::Negative => false,
            S::Positive | S::Zero => true,
        };

        let step_iva = || match twod_det(p1, q2, q1) {
            S::Positive => false,
            S::Negative | S::Zero => true,
        };

        let step_ivb = || match twod_det(p1, p2, r1) {
            S::Negative => false,
            S::Positive | S::Zero => step_va(),
        };

        let step_ivc = || match twod_det(q2, r2, r1) {
            S::Negative => false,
            S::Positive | S::Zero => step_vb(),
        };

        let step_ivd = || match twod_det(r1, p1, p2) {
            S::Negative => false,
            S::Positive | S::Zero => true,
        };

        let step_ive = || match twod_det(q1, r1, q2) {
            S::Negative => false,
            S::Positive | S::Zero => step_vc(),
        };

        let step_iiia = || match twod_det(p1, p2, q1) {
            S::Positive | S::Zero => step_iva(),
            S::Negative => step_ivb(),
        };

        let step_iiib = || match twod_det(p1, q2, q1) {
            S::Negative | S::Zero => step_ivc(),
            S::Positive => false,
        };

        let step_iiic = || match twod_det(q1, r1, r2) {
            S::Positive | S::Zero => step_ivd(),
            S::Negative => step_ive(),
        };

        let step_iia = || match twod_det(q2, r2, q1) {
            S::Positive | S::Zero => step_iiia(),
            S::Negative => step_iiib(),
        };

        let step_iib = || match twod_det(r2, p2, r1) {
            S::Negative => false,
            S::Positive | S::Zero => step_iiic(),
        };

        let step_i = || match twod_det(r2, p2, q1) {
            S::Positive | S::Zero => step_iia(),
            S::Negative => step_iib(),
        };

        step_i()
    }

    // compare ccw_p (index 0) to each of the pairs pq qr and rp
    #[expect(
        clippy::unnested_or_patterns,
        reason = "clarity from original material"
    )]
    match test_vertex_wrt_sides(ccw_two, p) {
        // impossible, colinear triangle
        [S::Zero, S::Zero, S::Zero] => false,
        // p inside triangle
        [S::Positive, S::Positive, S::Positive]
        // on a vertex
        | [S::Zero, S::Zero, _]
        | [S::Zero, _, S::Zero]
        | [_, S::Zero, S::Zero]
        // on an edge
        | [S::Zero, S::Positive, S::Positive]
        | [S::Positive, S::Zero, S::Positive]
        | [S::Positive, S::Positive, S::Zero] => true,
        // impossible, geometric reasons
        [S::Negative | S::Zero, S::Negative | S::Zero, S::Negative | S::Zero] => false,

        // interior of R1
        // do not need to permute the vertices
        a @ [S::Positive, S::Positive, S::Negative] => interior_r1(ccw_one, ccw_two, &a),
        // interior of R2
        // do not need to permute the vertices
        a @ ([S::Positive, S::Negative, S::Negative]
        // boundary of R2
        // do not need to permute the vertices
        | [S::Positive, S::Negative, S::Zero]
        | [S::Positive, S::Zero, S::Negative] )=> interior_boundary_r2(ccw_one, ccw_two, &a),

        // interior of an edge region
        // needs to be rotated by two
        a@ [S::Positive, S::Negative, S::Positive] => {
            let permutedtwo = permute!(ccw_two, 2);
            interior_r1(ccw_one, permutedtwo, &a)
        }

        // interior of a triangle region
        // needs to be rotated by two
        a@ ( [S::Negative, S::Negative, S::Positive]
        // boundary of a triangle region
        | [S::Negative, S::Zero, S::Positive]
        | [S::Zero, S::Negative, S::Positive] ) => {
            let permutedtwo = permute!(ccw_two, 2);
            interior_boundary_r2(ccw_one, permutedtwo, &a)
        }

        // interior of an edge region
        // needs to be rotated by one
        a @ [S::Negative, S::Positive, S::Positive] => {
            // TODO: could be permute 1, that goes for all of these
            let permutedtwo = permute!(ccw_two, 1);
            interior_r1(ccw_one, permutedtwo, &a)
        }
        // interior of a triangle region
        // needs to be rotated by one
        a@( [S::Negative, S::Positive, S::Negative]
        // boundary of a triangle region
        | [S::Zero, S::Positive, S::Negative]
        | [S::Negative, S::Positive, S::Zero]) => {
            let permutedtwo = permute!(ccw_two, 1);
            interior_boundary_r2(ccw_one, permutedtwo, &a)
        }
    }
}
