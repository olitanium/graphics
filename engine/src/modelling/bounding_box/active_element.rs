use std::array;

use graphics::linear_algebra::Vector;
use graphics::vertex::Vertex;

use crate::modelling::advanced_vertex::ExtendedVertex;
use crate::modelling::bounding_box::BoundingBoxElement;
use crate::modelling::{BoundingBox, Volume};
use crate::transform::Transform;

impl<V: ExtendedVertex<DIMS>, const DIMS: usize, T> BoundingBoxElement<V, DIMS, T>
where
    Vector<DIMS>: Volume<DIMS>,
{
    pub fn overlaps_bounding_box_single<P: Transform<Vector<DIMS>>>(
        &self,
        self_transform: &P,
        other: &Self,
        other_transform: &P,
    ) -> bool {
        let self_lower_bound = self_transform.transform(&self.lower_bound);
        let self_upper_bound = self_transform.transform(&self.upper_bound);
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

    pub fn encloses_point_and_children(
        &self,
        bounding_box: &BoundingBox<V, DIMS, T>,
        point: Vector<DIMS>,
    ) -> bool
    where
        [(); Vector::<DIMS>::ELEMENT_COUNT]:,
    {
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

    pub fn overlaps_bounding_box_and_children<
        'a,
        'b,
        P: Transform<Vector<DIMS>>,
        F: Fn(&mut T),
        O: Fn(&mut T),
    >(
        this: &'a Self,
        this_boundingbox: &'a BoundingBox<V, DIMS, T>,
        this_transform: &P,
        this_alter_meta: &F,
        other: &'b Self,
        other_boundingbox: &'b BoundingBox<V, DIMS, T>,
        other_transform: &P,
        depth: usize,
        other_alter_meta: &O,
    ) -> impl Iterator<
        Item = (
            &'a BoundingBoxElement<V, DIMS, T>,
            &'b BoundingBoxElement<V, DIMS, T>,
        ),
    >
    where
        [(); Vector::<DIMS>::ELEMENT_COUNT]:,
    {
        let is_overlap = this.overlaps_bounding_box_single(this_transform, other, other_transform);

        if is_overlap {
            this_alter_meta(&mut *this.meta.borrow_mut());
            other_alter_meta(&mut *other.meta.borrow_mut());
            match (
                this.get_children(this_boundingbox),
                other.get_children(other_boundingbox),
            ) {
                (Some([thisleft, thisright]), Some([otherleft, otherright])) => [
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
                        this_alter_meta,
                        new_other,
                        other_boundingbox,
                        other_transform,
                        depth + 1,
                        other_alter_meta,
                    )
                })
                .into_iter()
                .flatten()
                .collect(),
                (None, Some([otherleft, otherright])) => [(this, otherleft), (this, otherright)]
                    .map(|(new_this, new_other)| {
                        BoundingBoxElement::overlaps_bounding_box_and_children(
                            new_this,
                            this_boundingbox,
                            this_transform,
                            this_alter_meta,
                            new_other,
                            other_boundingbox,
                            other_transform,
                            depth + 1,
                            other_alter_meta,
                        )
                    })
                    .into_iter()
                    .flatten()
                    .collect(),
                (Some([thisleft, thisright]), None) => [(thisleft, other), (thisright, other)]
                    .map(|(new_this, new_other)| {
                        BoundingBoxElement::overlaps_bounding_box_and_children(
                            new_this,
                            this_boundingbox,
                            this_transform,
                            this_alter_meta,
                            new_other,
                            other_boundingbox,
                            other_transform,
                            depth + 1,
                            other_alter_meta,
                        )
                    })
                    .into_iter()
                    .flatten()
                    .collect(),
                (None, None) => {
                    vec![(this, other)]
                }
            }
        } else {
            vec![]
        }
        .into_iter()
    }

    pub fn get_children<'a>(
        &self,
        bounding_box: &'a BoundingBox<V, DIMS, T>,
    ) -> Option<[&'a Self; 2]>
    where
        [(); Vector::<DIMS>::ELEMENT_COUNT]:,
    {
        self.children
            .map(|arr_children| arr_children.map(|index| bounding_box.get(index.get())))
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
