use std::array;
use std::cell::RefMut;

use graphics::linear_algebra::Vector;
use graphics::vertex::Vertex;

use super::element::ChildrenOrMeta;
use crate::modelling::BoundingBox;
use crate::modelling::advanced_vertex::ExtendedVertex;
use crate::modelling::bounding_box::BoundingBoxElement;
use crate::transform::Transform;

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
                .into_iter()
                .map(|arr| {
                    let vec = Vector::new(arr);
                    pose.transform(&vec)
                })
                .collect();

        const EXPECT_MESSAGE: &str = "all_vertices.len() != 0  due to permutations returning many \
                                      results as DIMS != 0. Vector<DIMS>: Volume<DIMS>";

        let upper_bounds =
            array::from_fn(|index| all_vertices.iter().map(|vec| vec[index]).reduce(f32::max))
                .transpose()
                .expect(EXPECT_MESSAGE);
        let lower_bounds =
            array::from_fn(|index| all_vertices.iter().map(|vec| vec[index]).reduce(f32::min))
                .transpose()
                .expect(EXPECT_MESSAGE);

        (Vector::new(lower_bounds), Vector::new(upper_bounds))
    }

    pub fn overlaps_bounding_box_single<U, V2: ExtendedVertex<DIMS>, P: Transform<Vector<DIMS>>>(
        &self,
        self_transform: &P,
        other: &BoundingBoxElement<V2, DIMS, U>,
        other_transform: &P,
    ) -> bool {
        // let self_lower_bound = self_transform.transform(&self.lower_bound);
        // let self_upper_bound = self_transform.transform(&self.upper_bound);
        // let other_lower_bound = other_transform.transform(&other.lower_bound);
        // let other_upper_bound = other_transform.transform(&other.upper_bound);

        let (self_lower_bound, self_upper_bound) = self.transform_by(self_transform);
        let (other_lower_bound, other_upper_bound) = other.transform_by(other_transform);

        let lines: [_; DIMS] = array::from_fn(|index| {
            // let one_lb = f32::min(self_lower_bound[index], self_upper_bound[index]);
            // let one_ub = f32::max(self_lower_bound[index], self_upper_bound[index]);
            // let two_lb = f32::min(other_lower_bound[index], other_upper_bound[index]);
            // let two_ub = f32::max(other_lower_bound[index], other_upper_bound[index]);

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
    ) -> bool
    where
        Vector<DIMS>: ExtendedVertex<DIMS>,
        [(); <Vector<DIMS> as Vertex>::ELEMENT_COUNT]:,
    {
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

    pub fn overlaps_bounding_box_and_children<
        'a,
        'b,
        V2: ExtendedVertex<DIMS>,
        U,
        P: Transform<Vector<DIMS>>,
        F: FnMut(&mut T, &mut U),
    >(
        this: &'a Self,
        this_boundingbox: &'a BoundingBox<V, DIMS, T>,
        this_transform: &P,
        other: &'b BoundingBoxElement<V2, DIMS, U>,
        other_boundingbox: &'b BoundingBox<V2, DIMS, U>,
        other_transform: &P,
        alter_meta: &mut F,
    ) -> bool
    where
        Vector<DIMS>: ExtendedVertex<DIMS>,
        [(); <Vector<DIMS> as Vertex>::ELEMENT_COUNT]:,
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
                .any(|(new_this, new_other)| {
                    BoundingBoxElement::overlaps_bounding_box_and_children(
                        new_this,
                        this_boundingbox,
                        this_transform,
                        new_other,
                        other_boundingbox,
                        other_transform,
                        alter_meta,
                    )
                }),
                (
                    InnerChildrenOrMeta::Meta(_),
                    InnerChildrenOrMeta::Children([otherleft, otherright]),
                ) => [(this, otherleft), (this, otherright)].into_iter().any(
                    |(new_this, new_other)| {
                        BoundingBoxElement::overlaps_bounding_box_and_children(
                            new_this,
                            this_boundingbox,
                            this_transform,
                            new_other,
                            other_boundingbox,
                            other_transform,
                            alter_meta,
                        )
                    },
                ),
                (
                    InnerChildrenOrMeta::Children([thisleft, thisright]),
                    InnerChildrenOrMeta::Meta(_),
                ) => [(thisleft, other), (thisright, other)].into_iter().any(
                    |(new_this, new_other)| {
                        BoundingBoxElement::overlaps_bounding_box_and_children(
                            new_this,
                            this_boundingbox,
                            this_transform,
                            new_other,
                            other_boundingbox,
                            other_transform,
                            alter_meta,
                        )
                    },
                ),
                (InnerChildrenOrMeta::Meta(mut lhs), InnerChildrenOrMeta::Meta(mut rhs)) => {
                    alter_meta(&mut *lhs, &mut *rhs);
                    true
                }
            }
        } else {
            false
        }
    }

    pub(crate) fn get_children<'a>(
        &'a self,
        bounding_box: &'a BoundingBox<V, DIMS, T>,
    ) -> InnerChildrenOrMeta<'a, V, DIMS, T>
    where
        Vector<DIMS>: ExtendedVertex<DIMS>,
        [(); <Vector<DIMS> as Vertex>::ELEMENT_COUNT]:,
    {
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
    assert!(one_ub > one_lb);
    assert!(two_ub > two_lb);

    let fail = (one_ub < two_lb) || (two_ub < one_lb);
    !fail
}

pub fn permutations<T: Clone, const DIMS: usize>(
    input_vec_1: [T; DIMS],
    input_vec_2: [T; DIMS],
) -> Vec<[T; DIMS]> {
    (0..(1_usize << DIMS))
        .map(|bin| {
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
        .collect()
}
mod test {

    #[test]
    fn test_line_overlap() {
        use crate::modelling::bounding_box::active_element::{TestBoxVertex, line_overlap};

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
        use crate::modelling::bounding_box::active_element::permutations;

        let vec1 = [1, 1, 1];
        let vec2 = [0, 0, 0];

        assert_eq!(
            permutations(vec1, vec2),
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
