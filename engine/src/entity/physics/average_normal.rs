use std::iter;

use graphics::linear_algebra::{UnitVector, Vector, vector};
use zip_with::ArrayZipWith;

use crate::modelling::CubicVertex;

fn arrange(slice: &mut [(Vector<3>, UnitVector<3>)], mid_point: Vector<3>) {
    slice.sort_by_key(|(pos, _)| {
        pos.inner()
            .each_ref()
            .zip_with(mid_point.inner().each_ref(), f32::total_cmp)
    });
}

pub(super) fn split(
    slice: &mut [(Vector<3>, UnitVector<3>)],
    mid_point: Vector<3>,
) -> impl Iterator<Item = &mut [(Vector<3>, UnitVector<3>)]> {
    arrange(slice, mid_point);

    let [mx, my, mz] = mid_point.into_inner();

    // split on x,
    // split on y,
    // split on z,

    slice
        .split_inclusive_mut(move |(vec, _)| vec.x() <= mx)
        .flat_map(move |slice| slice.split_inclusive_mut(move |(vec, _)| vec.y() <= my))
        .flat_map(move |slice| slice.split_inclusive_mut(move |(vec, _)| vec.z() <= mz))
}

pub(super) fn split_into_eight(
    slice: &mut [(Vector<3>, UnitVector<3>)],
    upper_bound: Vector<3>,
    lower_bound: Vector<3>,
) -> impl Iterator<Item = (Vector<3>, Vector<3>, &mut [(Vector<3>, UnitVector<3>)])> {
    let mid_point = 0.5 * (upper_bound + lower_bound);

    let [ux, uy, uz] = upper_bound.into_inner();
    let [mx, my, mz] = mid_point.into_inner();
    let [lx, ly, lz] = lower_bound.into_inner();

    let out_iter = split(slice, mid_point);

    let upper_lower_bounds = [
        (vector![mx, my, mz], vector![lx, ly, lz]),
        (vector![mx, my, uz], vector![lx, ly, mz]),
        (vector![mx, uy, mz], vector![lx, my, lz]),
        (vector![mx, uy, uz], vector![lx, my, mz]),
        (vector![ux, my, mz], vector![mx, ly, lz]),
        (vector![ux, my, uz], vector![mx, ly, mz]),
        (vector![ux, uy, mz], vector![mx, my, lz]),
        (vector![ux, uy, uz], vector![mx, my, mz]),
    ];

    iter::zip(upper_lower_bounds, out_iter).map(|((upper, lower), slice)| (upper, lower, slice))
}

/// ```text
/// get all vertices (positions, normals) individually.
/// find axis aligned maximums and minimums.
/// count number of vertices
///
/// recursive (count_in_previous, partition)
///     split into eight - (in 3D)
///     partition vertices (maybe by shuffling the pieces around)
///     for each partition
///         count number_of_vertices
///         if number_of_vertices == 0 {
///             this partition counts for None;
///         }
///         if number_of_vertices == count_in_previous {
///             this partition counts for Some(average(partition));
///         }
///         otherwise:
///             this partition counts for Some(recursive(number_of_vertices, this_partition));
///     return average the values of each partition
/// ```
pub(super) fn get_average_normal<V: CubicVertex + PartialEq>(
    this: &mut [[V; 3]],
) -> Option<UnitVector<3>> {
    let mut x = this
        .iter()
        .flat_map(|tri| iter::zip(V::multi_position(tri), V::multi_normal(tri)))
        .collect::<Vec<(Vector<3>, _)>>();

    let max = try {
        let max_x = x.iter().map(|(pos, _)| pos.x()).max_by(f32::total_cmp)?;
        let max_y = x.iter().map(|(pos, _)| pos.y()).max_by(f32::total_cmp)?;
        let max_z = x.iter().map(|(pos, _)| pos.z()).max_by(f32::total_cmp)?;
        vector![max_x, max_y, max_z]
    }?; // returns None if no vertices

    let min = try {
        let min_x = x.iter().map(|(pos, _)| pos.x()).min_by(f32::total_cmp)?;
        let min_y = x.iter().map(|(pos, _)| pos.y()).min_by(f32::total_cmp)?;
        let min_z = x.iter().map(|(pos, _)| pos.z()).min_by(f32::total_cmp)?;
        vector![min_x, min_y, min_z]
    }?; // returns None if no vertices

    let number_vertices = x.len();

    fn recursion(
        part: &mut [(Vector<3>, UnitVector<3>)],
        ub: Vector<3>,
        lb: Vector<3>,
        prev_length: usize,
    ) -> Option<UnitVector<3>> {
        split_into_eight(part, ub, lb)
            .filter_map(|(new_upper, new_lower, partition)| {
                let number_this_part = partition.len();

                if number_this_part == 0 {
                    return None;
                } else if number_this_part == prev_length {
                    let vec: Vector<3> = partition.iter().map(|(_, unit)| unit.v()).sum();
                    vec.try_normalize()
                } else {
                    recursion(partition, new_upper, new_lower, number_this_part)
                }
            })
            .map(UnitVector::v)
            .sum::<Vector<_>>()
            .try_normalize()
    }

    recursion(&mut x, max, min, number_vertices)
}

#[cfg(test)]
mod test {
    use geometry::NoTransform;
    use graphics::linear_algebra::{UnitVector, Vector, vector};
    use graphics::vertex::Vertex;

    use super::{arrange, split, split_into_eight};
    use crate::entity::physics::average_normal::get_average_normal;
    use crate::modelling::{CubicVertex, ExtendedVertex};

    #[test]
    fn test_arrange() {
        let mut test = [
            vector![-1.0, -1.0, -1.0],
            vector!(-1.0, -1.0, 1.0),
            vector!(-1.0, 1.0, -1.0),
            vector!(-1.0, 1.0, 1.0),
            vector!(1.0, -1.0, -1.0),
            vector!(1.0, -1.0, 1.0),
            vector!(1.0, 1.0, -1.0),
            vector!(1.0, 1.0, 1.0_f32),
        ]
        .map(|x| (x, UnitVector::X));

        test.reverse();

        arrange(&mut test, Vector::ZERO);

        assert_eq!(
            test,
            [
                vector![-1.0, -1.0, -1.0],
                vector!(-1.0, -1.0, 1.0),
                vector!(-1.0, 1.0, -1.0),
                vector!(-1.0, 1.0, 1.0),
                vector!(1.0, -1.0, -1.0),
                vector!(1.0, -1.0, 1.0),
                vector!(1.0, 1.0, -1.0),
                vector!(1.0, 1.0, 1.0_f32),
            ]
            .map(|x| (x, UnitVector::X))
        )
    }

    #[test]
    fn test_split() {
        let mut test = [
            vector![-1.0, -1.0, -1.0],
            vector!(-1.0, -1.0, 1.0),
            vector!(-1.0, 1.0, -1.0),
            vector!(-1.0, 1.0, 1.0),
            vector!(1.0, -1.0, -1.0),
            vector!(1.0, -1.0, 1.0),
            vector!(1.0, 1.0, -1.0),
            vector!(1.0, 1.0, 1.0),
        ]
        .map(|x| (x, UnitVector::X));

        test.reverse();

        let out: Vec<_> = split(&mut test, Vector::ZERO).collect();

        assert_eq!(
            out,
            [
                &[(vector![-1.0, -1.0, -1.0], UnitVector::X)],
                &[(vector!(-1.0, -1.0, 1.0), UnitVector::X)],
                &[(vector!(-1.0, 1.0, -1.0), UnitVector::X)],
                &[(vector!(-1.0, 1.0, 1.0), UnitVector::X)],
                &[(vector!(1.0, -1.0, -1.0), UnitVector::X)],
                &[(vector!(1.0, -1.0, 1.0), UnitVector::X)],
                &[(vector!(1.0, 1.0, -1.0), UnitVector::X)],
                &[(vector!(1.0, 1.0, 1.0_f32), UnitVector::X)],
            ]
        )
    }

    #[test]
    // TODO: add all the rows
    fn test_split_into_eight() {
        let mut test = [
            vector![-1.0, -1.0, -1.0],
            vector!(-1.0, -1.0, 1.0),
            vector!(-1.0, 1.0, -1.0),
            vector!(-1.0, 1.0, 1.0),
            vector!(1.0, -1.0, -1.0),
            vector!(1.0, -1.0, 1.0),
            vector!(1.0, 1.0, -1.0),
            vector!(1.0, 1.0, 1.0_f32),
        ]
        .map(|x| (x, UnitVector::X));

        let mut iter =
            split_into_eight(&mut test, vector!(1.0, 1.0, 1.0), vector![-1.0, -1.0, -1.0]);

        assert_eq!(
            iter.next().unwrap(),
            (
                Vector::ZERO,
                vector![-1.0, -1.0, -1.0_f32],
                [(vector![-1.0, -1.0, -1.0_f32], UnitVector::X)].as_mut_slice()
            ),
        )
    }

    #[test]
    fn test_get_average() {
        #[derive(Debug, Clone, Copy, PartialEq)]
        struct TestVertex(Vector<3>);

        impl Vertex for TestVertex {
            type const ELEMENT_COUNT: usize = 0;

            fn offsets() -> [usize; Self::ELEMENT_COUNT] {
                []
            }

            fn types_of() -> [graphics::types::VertexAttrType; Self::ELEMENT_COUNT] {
                []
            }
        }
        impl ExtendedVertex for TestVertex {
            type PreferredTransform<'a> = NoTransform;

            type const LEN: usize = 3;

            fn position(&self) -> Vector<{ Self::LEN }> {
                self.0
            }

            fn from_incomplete_triangle(
                triangle: [crate::modelling::IncompleteVertex<{ Self::LEN }>; 3],
            ) -> Option<[Self; 3]> {
                Some(triangle.map(|x| Self(x.position)))
            }
        }
        impl CubicVertex for TestVertex {
            fn triangle_normal(triangle: &[Self; 3]) -> UnitVector<3> {
                let [first, ..] = triangle;
                let unit = first.0.try_normalize().unwrap();
                unit
            }
        }

        let mut test = [
            [TestVertex(vector![1.0, 0.0, 0.0]); 3],
            [TestVertex(vector![-1.0, 0.0, 0.0]); 3],
            [TestVertex(vector![0.0, 1.0, 0.0]); 3],
            [TestVertex(vector![0.0, -1.0, 0.0]); 3],
            [TestVertex(vector![0.0, 0.0, 1.0]); 3],
            [TestVertex(vector![0.0, 0.0, -1.0]); 3],
        ];

        let out = get_average_normal(&mut test);

        assert_eq!(out, None);
        // assert_eq!(out, Some(UnitVector::Y))
    }
}
