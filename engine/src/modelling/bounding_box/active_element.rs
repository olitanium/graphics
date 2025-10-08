use std::{array, ops::Bound};

use graphics::linear_algebra::Vector;

use crate::{TransformBy, modelling::{BoundingBox, advanced_vertex::ExtendedVertex, bounding_box::BoundingBoxElement, cubic::geometry::{ApplyPose, Pose}}, transform::Transform};

impl<const DIMS: usize> BoundingBoxElement<DIMS> {
    pub fn overlaps_bounding_box_single<T: Transform<Vector<DIMS>>>(&self, self_transform: &T, other: &Self, other_transform: &T) -> bool {
        let self_lower_bound  = self_transform.transform(&self.lower_bound);
        let self_upper_bound  = self_transform.transform(&self.upper_bound);
        let other_lower_bound = other_transform.transform(&other.lower_bound);
        let other_upper_bound = other_transform.transform(&other.upper_bound);

        let lines: [_; DIMS] = array::from_fn(|index| {
            let one_lb = f32::min(self_lower_bound[index], self_upper_bound[index]);
            let one_ub = f32::max(self_lower_bound[index], self_upper_bound[index]);
            let two_lb = f32::min(other_lower_bound[index], other_upper_bound[index]);
            let two_ub = f32::max(other_lower_bound[index], other_upper_bound[index]);
            
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

        let is_in = offset_from_maximum.into_iter().all(|x| x >= 0.0)
            && offset_from_minimum.into_iter().all(|x| x >= 0.0);

        is_in
    }

    pub fn encloses_point_and_children<V: ExtendedVertex<DIMS>>(
        &self,
        bounding_box: &BoundingBox<V, DIMS>,
        point: Vector<DIMS>,
    ) -> bool {
        let is_in = self.encloses_point_single(point);

        match self.get_children(bounding_box) {
            Some([left, right]) => {
                is_in
                    && (left.encloses_point_and_children(bounding_box, point)
                        || right.encloses_point_and_children(bounding_box, point))
            }
            None => is_in,
        }
    }
}

impl<const DIMS: usize> BoundingBoxElement<DIMS> {
    pub fn overlaps_bounding_box_and_children<'c, 'b, V: ExtendedVertex<DIMS>, T: Transform<Vector<DIMS>>>(
        this: &'c Self,
        this_boundingbox: &'c BoundingBox<V, DIMS>,
        this_transform: &T,
        other: &'b Self,
        other_boundingbox: &'b BoundingBox<V, DIMS>,
        other_transform: &T,
        depth: usize,
    ) -> impl Iterator<Item = (&'c BoundingBoxElement<DIMS>, &'b BoundingBoxElement<DIMS>)> {
        let is_overlap = this.overlaps_bounding_box_single(this_transform, other, other_transform);

        if is_overlap {
            this.collision.set(true);
            other.collision.set(true);
            match (
                this.get_children(this_boundingbox),
                other.get_children(other_boundingbox),
            ) {
                (Some([thisleft, thisright]), Some([otherleft, otherright])) => {
                    eprintln!("{:depth$}fourway", "");
                    [
                        (thisleft, otherleft),
                        (thisleft, otherright),
                        (thisright, otherleft),
                        (thisright, otherright),
                    ]
                    .map(|(new_this, new_other)| {
                        BoundingBoxElement::overlaps_bounding_box_and_children(
                            new_this,
                            this_boundingbox,
                            this_transform,
                            new_other,
                            other_boundingbox,
                            other_transform,
                            depth + 1,
                        )
                    })
                    .into_iter()
                    .flatten()
                    .collect()
                }
                (None, Some([otherleft, otherright])) => {
                    eprintln!("{:depth$}self_leaf other_not", "");
                    [(this, otherleft), (this, otherright)]
                        .map(|(new_this, new_other)| {
                            BoundingBoxElement::overlaps_bounding_box_and_children(
                                new_this,
                                this_boundingbox,
                                this_transform,
                                new_other,
                                other_boundingbox,
                                other_transform,
                                depth + 1,
                            )
                        })
                        .into_iter()
                        .flatten()
                        .collect()
                }
                (Some([thisleft, thisright]), None) => {
                    eprintln!("{:depth$}this_not other_leaf", "");
                    [(thisleft, other), (thisright, other)]
                        .map(|(new_this, new_other)| {
                            BoundingBoxElement::overlaps_bounding_box_and_children(
                                new_this,
                                this_boundingbox,
                                this_transform,
                                new_other,
                                other_boundingbox,
                                other_transform,
                                depth + 1,
                            )
                        })
                        .into_iter()
                        .flatten()
                        .collect()
                }
                (None, None) => {
                    eprintln!("{:depth$}leaf!", "");
                    vec![(this, other)]
                }
            }
        } else {
            vec![]
        }
        .into_iter()
    }
    
    pub fn get_children<'b, V: ExtendedVertex<DIMS>> (
        &self,
        bounding_box: &'b BoundingBox<V, DIMS>,
    ) -> Option<[&'b Self; 2]>
    {
        self.children
            .map(|arr_children| arr_children.map(|index| bounding_box.get(index)))
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

fn cartesian_product<'a, 'b, T, U>(
    slice_one: &'a [T],
    slice_two: &'b [U],
) -> impl Iterator<Item = (&'a T, &'b U)> {
    slice_one
        .into_iter()
        .flat_map(move |value1| slice_two.into_iter().map(move |value2| (value1, value2)))
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
}