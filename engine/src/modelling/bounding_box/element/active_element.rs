use std::cell::RefMut;
use std::fmt::Debug;
use std::{array, cmp};

use geometry::{Transform, TransformBy};
use graphics::linear_algebra::Vector;
use zip_with::ArrayZipWith;

use super::ChildrenOrMeta;
use crate::modelling::BoundingBox;
use crate::modelling::bounding_box::BoundingBoxElement;
use crate::modelling::bounding_box::element::{make_bound, make_bound_inner};
use crate::modelling::extended_vertex::ExtendedVertex;

pub(crate) enum InnerChildrenOrMeta<'a, V: ExtendedVertex<DIMS>, const DIMS: usize, T> {
    Meta(RefMut<'a, T>),
    Children([&'a BoundingBoxElement<V, DIMS, T>; 2]),
}

impl<V: ExtendedVertex<DIMS>, const DIMS: usize, T> BoundingBoxElement<V, DIMS, T> {
    pub fn transform_by<P: Transform<Vector<DIMS>>>(
        &self,
        pose: &P,
    ) -> (Vector<DIMS>, Vector<DIMS>) {
        let all_vertices: Vec<_> =
            permutations(self.lower_bound.into_inner(), self.upper_bound.into_inner())
                .map(|arr| {
                    let vec = Vector::new(arr);
                    pose.transform(&vec)
                })
                .collect();

        const EXPECT_MESSAGE: &str = "all_vertices.len() != 0  due to permutations returning many \
                                      results as DIMS != 0. Vector<DIMS>: Volume<DIMS>";

        let upper_bounds = array::from_fn(|index| {
            all_vertices
                .iter()
                .map(|vec| vec.get(index).copied())
                .reduce(|lhs, rhs| Option::zip(lhs, rhs).map(|(a, b)| f32::max(a, b)))
                .flatten()
        })
        .transpose()
        .expect(EXPECT_MESSAGE);

        let lower_bounds_new = all_vertices
            .iter()
            .map(|x| x.into_inner())
            .reduce(|vec1, vec2| vec1.zip_with(vec2, |lhs, rhs| f32::min(lhs, rhs)))
            .expect(EXPECT_MESSAGE);

        let lower_bounds = array::from_fn(|index| {
            all_vertices
                .iter()
                .map(|vec| vec.get(index).copied())
                .reduce(|lhs, rhs| Option::zip(lhs, rhs).map(|(a, b)| f32::min(a, b)))
                .flatten()
        })
        .transpose()
        .expect(EXPECT_MESSAGE);

        assert_eq!(Vector::new(lower_bounds), Vector::new(lower_bounds_new));

        (Vector::new(lower_bounds), Vector::new(upper_bounds))
    }

    pub fn overlaps_bounding_box_single<
        U,
        V2: ExtendedVertex<DIMS>,
        P: Transform<Vector<DIMS>>,
        P2: Transform<Vector<DIMS>>,
    >(
        &self,
        self_transform: &P,
        other: &BoundingBoxElement<V2, DIMS, U>,
        other_transform: &P2,
    ) -> bool {
        // let self_lower_bound = self_transform.transform(&self.lower_bound);
        // let self_upper_bound = self_transform.transform(&self.upper_bound);
        // let other_lower_bound = other_transform.transform(&other.lower_bound);
        // let other_upper_bound = other_transform.transform(&other.upper_bound);

        let (self_lower_bound, self_upper_bound) = self.transform_by(self_transform);
        let (other_lower_bound, other_upper_bound) = other.transform_by(other_transform);

        let lines: [_; DIMS] = array::from_fn(|index| {
            let one_lb = self_lower_bound[index];
            let one_ub = self_upper_bound[index];
            let two_lb = other_lower_bound[index];
            let two_ub = other_upper_bound[index];

            TestBoxVertex {
                one_lb,
                one_ub,
                two_lb,
                two_ub,
            }
        });

        // TODO: MultiZip self, and other.

        lines.into_iter().all(line_overlap)
    }

    pub fn encloses_point_single(&self, point: Vector<DIMS>) -> bool {
        let offset_from_minimum = point - self.lower_bound; // if in, all positive
        let offset_from_maximum = self.upper_bound - point; // if in, all positive

        offset_from_maximum.into_iter().all(|x| x >= 0.0)
            && offset_from_minimum.into_iter().all(|x| x >= 0.0)
    }

    pub fn encloses_point_and_children(
        &self,
        bounding_box: &BoundingBox<V, DIMS, T>,
        point: Vector<DIMS>,
    ) -> bool {
        let is_in = self.encloses_point_single(point);

        match self.get_children(bounding_box) {
            InnerChildrenOrMeta::Children([left, right]) => {
                is_in
                    && (left.encloses_point_and_children(bounding_box, point)
                        || right.encloses_point_and_children(bounding_box, point))
            }
            _ => is_in,
        }
    }
}

impl<V: ExtendedVertex<3>, T> BoundingBoxElement<V, 3, T> {
    pub fn intersects_triangle<P>(&self, transform: &P, triangle: &[Vector<3>; 3]) -> bool
    where
        P: Transform<Vector<3>>,
    {
        let Some((other_lower_bound, other_upper_bound)) = make_bound_inner(triangle) else {
            return false;
        };

        let (self_lower_bound, self_upper_bound) = self.transform_by(transform);

        let lines: [_; 3] = array::from_fn(|index| {
            let one_lb = self_lower_bound[index];
            let one_ub = self_upper_bound[index];
            let two_lb = other_lower_bound[index];
            let two_ub = other_upper_bound[index];

            TestBoxVertex {
                one_lb,
                one_ub,
                two_lb,
                two_ub,
            }
        });

        // TODO: MultiZip self, and other.

        lines.into_iter().all(line_overlap)
    }

    pub fn overlaps_bounding_box_and_children<
        'a,
        'b,
        V2: ExtendedVertex<3>,
        U,
        P: Transform<V> + Transform<Vector<3>>,
        P2: Transform<V2> + Transform<Vector<3>>,
        F: FnMut(&mut T, &mut U),
    >(
        this: &'a Self,
        this_boundingbox: &'a BoundingBox<V, 3, T>,
        this_transform: &P,
        other: &'b BoundingBoxElement<V2, 3, U>,
        other_boundingbox: &'b BoundingBox<V2, 3, U>,
        other_transform: &P2,
        alter_meta: &mut F,
        accumulator: &mut Vec<([V; 3], [V2; 3])>,
    )
    // -> Vec<(&'a [V; 3], &'b [V2; 3])>)
    {
        let is_overlap = this.overlaps_bounding_box_single(this_transform, other, other_transform);

        if is_overlap {
            match (
                this.get_children(this_boundingbox),
                other.get_children(other_boundingbox),
            ) {
                (
                    InnerChildrenOrMeta::Children([thisleft, thisright]),
                    InnerChildrenOrMeta::Children([otherleft, otherright]),
                ) => [
                    (thisleft, otherleft),
                    (thisleft, otherright),
                    (thisright, otherleft),
                    (thisright, otherright),
                ]
                .into_iter()
                .for_each(|(new_this, new_other)| {
                    Self::overlaps_bounding_box_and_children(
                        new_this,
                        this_boundingbox,
                        this_transform,
                        new_other,
                        other_boundingbox,
                        other_transform,
                        alter_meta,
                        accumulator,
                    )
                }),
                (
                    InnerChildrenOrMeta::Meta(_),
                    InnerChildrenOrMeta::Children([otherleft, otherright]),
                ) => [(this, otherleft), (this, otherright)]
                    .into_iter()
                    .for_each(|(new_this, new_other)| {
                        Self::overlaps_bounding_box_and_children(
                            new_this,
                            this_boundingbox,
                            this_transform,
                            new_other,
                            other_boundingbox,
                            other_transform,
                            alter_meta,
                            accumulator,
                        )
                    }),
                (
                    InnerChildrenOrMeta::Children([thisleft, thisright]),
                    InnerChildrenOrMeta::Meta(_),
                ) => [(thisleft, other), (thisright, other)]
                    .into_iter()
                    .for_each(|(new_this, new_other)| {
                        Self::overlaps_bounding_box_and_children(
                            new_this,
                            this_boundingbox,
                            this_transform,
                            new_other,
                            other_boundingbox,
                            other_transform,
                            alter_meta,
                            accumulator,
                        )
                    }),
                (InnerChildrenOrMeta::Meta(mut lhs), InnerChildrenOrMeta::Meta(mut rhs)) => {
                    alter_meta(&mut *lhs, &mut *rhs);

                    let this_triangles = this.get_triangles(this_boundingbox);
                    let other_triangles = other.get_triangles(other_boundingbox);

                    // compare all triangles in this with all triangles in other.

                    let other_iter = other_triangles
                        .iter()
                        .map(|(_, tri)| {
                            let tri_transformed =
                                tri.each_ref().map(|vert| other_transform.transform(vert));
                            let positions = tri.each_ref().map(|vert| vert.position());
                            (tri_transformed, positions)
                        })
                        .filter(|(triangle, pos)| this.intersects_triangle(this_transform, pos));

                    this_triangles
                        .iter()
                        .filter_map(|(_, tri)| {
                            let new_this =
                                tri.each_ref().map(|vert| this_transform.transform(vert));
                            let this_pos = new_this.each_ref().map(|vert| vert.position());

                            if other.intersects_triangle(other_transform, &this_pos) {
                                Some(other_iter.clone().flat_map(move |(new_other, other_pos)| {
                                    triangle_intersection::intersect3(this_pos, other_pos)
                                        .then_some((new_this, new_other))
                                }))
                            } else {
                                None
                            }
                        })
                        .flatten()
                        .collect_into(accumulator);
                }
            }
        }
    }
}

impl<V: ExtendedVertex<DIMS>, const DIMS: usize, T> BoundingBoxElement<V, DIMS, T> {
    pub(crate) fn get_children<'a>(
        &'a self,
        bounding_box: &'a BoundingBox<V, DIMS, T>,
    ) -> InnerChildrenOrMeta<'a, V, DIMS, T> {
        match &self.children_or_meta {
            ChildrenOrMeta::Meta(cell) => InnerChildrenOrMeta::Meta(cell.borrow_mut()),
            ChildrenOrMeta::Children(children) => {
                InnerChildrenOrMeta::Children(children.map(|index| bounding_box.get(index)))
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct TestBoxVertex {
    one_lb: f32,
    one_ub: f32,
    two_lb: f32,
    two_ub: f32,
}

fn line_overlap(
    TestBoxVertex {
        one_lb,
        one_ub,
        two_lb,
        two_ub,
    }: TestBoxVertex,
) -> bool {
    assert!(one_ub >= one_lb, "{one_ub} >= {one_lb}");
    assert!(two_ub >= two_lb, "{two_ub} >= {two_lb}");

    let fail = (one_ub < two_lb) || (two_ub < one_lb);
    !fail
}

pub fn permutations<T: Clone, const DIMS: usize>(
    input_vec_1: [T; DIMS],
    input_vec_2: [T; DIMS],
) -> impl Iterator<Item = [T; DIMS]> {
    (0..(1_usize << DIMS)).map(move |bin| {
        array::from_fn(|index| {
            let mask = 1 << index;
            let first = (bin & mask) != 0;
            if first {
                input_vec_1[index].clone()
            } else {
                input_vec_2[index].clone()
            }
        })
    })
}
mod test {

    #[test]
    fn test_line_overlap() {
        use super::{TestBoxVertex, line_overlap};

        let input = TestBoxVertex {
            one_lb: -1.0,
            one_ub: 1.0,
            two_lb: 0.5,
            two_ub: 2.0,
        };

        assert!(line_overlap(input))
    }

    #[test]
    fn test_permuations() {
        use super::permutations;

        let vec1 = [1, 1, 1];
        let vec2 = [0, 0, 0];

        assert_eq!(
            permutations(vec1, vec2).collect::<Vec<_>>(),
            vec![
                [0, 0, 0],
                [1, 0, 0],
                [0, 1, 0],
                [1, 1, 0],
                [0, 0, 1],
                [1, 0, 1],
                [0, 1, 1],
                [1, 1, 1],
            ]
        )
    }
}
