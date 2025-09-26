use std::rc::Rc;
use std::{array, iter};

use array_vec::ArrayVec;
use graphics::linear_algebra::Vector;

use crate::modelling::advanced_vertex::{ExtendedVertex, IncompleteVertex};
use crate::modelling::cubic::Mesh;
use crate::modelling::cubic::geometry::Pose;
use crate::modelling::cubic::material::Material;
use crate::modelling::test_models::vertex_array_cube;
use crate::modelling::{Cubic, CubicVertex, SimpleVertex};

#[derive(Debug, Clone)]
pub struct BoundingBox<V: ExtendedVertex<N>, const N: usize> {
    triangles: Vec<[V; 3]>,
    inner: Vec<BoundingBoxElement<N>>,
}

#[derive(Debug, Clone)]
struct BoundingBoxElement<const N: usize> {
    lower_bounds: Vector<N>,
    upper_bounds: Vector<N>,

    lower_bound_index: usize,
    upper_bound_index: usize,

    children: Option<[usize; 2]>,
}

fn make_bound<V: ExtendedVertex<N>, const N: usize>(
    vectors: &[V],
    func: fn(f32, f32) -> f32,
) -> Option<Vector<N>> {
    array::from_fn(|index| {
        vectors
            .into_iter()
            .map(|vec| vec.position().truncate::<N>()[index])
            .reduce(func)
    })
    .transpose()
    .map(Vector::new)
}

fn make_upper_bound<V: ExtendedVertex<N>, const N: usize>(vectors: &[V]) -> Option<Vector<N>> {
    make_bound(vectors, f32::max)
}

fn make_lower_bound<V: ExtendedVertex<N>, const N: usize>(vectors: &[V]) -> Option<Vector<N>> {
    make_bound(vectors, f32::min)
}

impl<V: ExtendedVertex<N>, const N: usize> BoundingBox<V, N> {
    fn get(&self, index: usize) -> &BoundingBoxElement<N> {
        &self.inner[index]
    }
}

impl<V: ExtendedVertex<N>, const N: usize> BoundingBox<V, N> {
    pub fn new_from_vec_indexes(
        vectors: Vec<V>,
        triangles: &[[usize; 3]],
        depth: usize,
    ) -> Option<Self> {
        let new_triangles: Option<Vec<_>> = triangles
            .into_iter()
            .map(|arr| arr.map(|index| vectors.get(index).copied()).transpose())
            .collect();
        let new_triangles = new_triangles?;
        Self::new_from_triangles(new_triangles, depth)
    }

    pub fn new_from_triangles(triangles: Vec<[V; 3]>, depth: usize) -> Option<Self> {
        let mut triangles_with_mid: Vec<_> = triangles.into_iter().map(
            |triangle| {
                let vertices = triangle.map(|vertex| vertex.position().truncate(/* currently required to force the vectors to the correct length */));
                let mut mid: Vector<_> = vertices.into_iter().sum();
                const ONE_THIRD: f32 = 1.0/3.0;
                mid = mid.scale(ONE_THIRD);

                (mid, triangle)
            }
        ).collect();

        let len_triangles_with_mid = triangles_with_mid.len();

        let mut so_far = Vec::new();

        let first_elem_index = BoundingBoxElement::new_from_many_inner(
            &mut so_far,
            &mut triangles_with_mid,
            0,
            len_triangles_with_mid,
            depth,
        )?;

        let discard_cached_mid = triangles_with_mid.into_iter().map(|(_, tri)| tri).collect();

        assert_eq!(first_elem_index, 0);

        Some(BoundingBox {
            triangles: discard_cached_mid,
            inner: so_far,
        })
    }

    pub fn encloses_point(&self, point: Vector<N>) -> bool {
        self.get(0).encloses_point_and_children(self, point)
    }
}

impl<const N: usize> BoundingBoxElement<N> {
    fn new_from_many_inner<V: ExtendedVertex<N>>(
        so_far: &mut Vec<Self>,
        triangles: &mut [(Vector<N>, [V; 3])],
        lower_bound_index: usize,
        upper_bound_index: usize,
        depth: usize,
    ) -> Option<usize> {
        let vectors: Vec<_> = triangles.into_iter().flat_map(|(_, arr)| *arr).collect();
        let upper_bounds = make_upper_bound(&vectors)?;
        let lower_bounds = make_lower_bound(&vectors)?;

        println!("lb: {lower_bound_index}, ub: {upper_bound_index}");
        assert!(lower_bound_index <= upper_bound_index);
        let destination_index = so_far.len();
        so_far.push(Self {
            lower_bound_index,
            upper_bound_index,
            lower_bounds,
            upper_bounds,
            children: None,
        });

        // vectors must be populated at this point
        if depth != 0 {
            so_far[destination_index].children = try {
                let dimensions = upper_bounds - lower_bounds;
                let (max_index, _) = dimensions
                    .into_iter()
                    .enumerate()
                    .max_by(|(_, size), (_, size2)| {
                        f32::partial_cmp(size, size2).expect(
                            "`vectors` has values, so has filled the previous step. No NAN in \
                             this vector",
                        )
                    })
                    .expect("input vector const N > 0");

                let centre_point = 0.5 * (upper_bounds + lower_bounds);

                // split on the dimension given by max_index
                triangles.sort_unstable_by(|(lhs, _), (rhs, _)| {
                    f32::partial_cmp(&lhs[max_index], &rhs[max_index])
                        .expect("no NAN in input vectors permitted")
                });
                let partition = triangles
                    .partition_point(|(centre, _)| centre[max_index] <= centre_point[max_index]);

                let (lower, upper) = triangles.split_at_mut(partition);

                let lower_bounding_box = Self::new_from_many_inner(
                    so_far,
                    lower,
                    lower_bound_index,
                    lower_bound_index + partition,
                    depth - 1,
                )?;
                let upper_bounding_box = Self::new_from_many_inner(
                    so_far,
                    upper,
                    lower_bound_index + partition,
                    upper_bound_index,
                    depth - 1,
                )?;

                [lower_bounding_box, upper_bounding_box]
            }
        };

        Some(destination_index)
    }

    pub fn encloses_point_single(&self, point: Vector<N>) -> bool {
        let offset_from_minimum = point - self.lower_bounds; // if in, all positive
        let offset_from_maximum = self.upper_bounds - point; // if in, all positive

        let is_in = offset_from_maximum.into_iter().all(|x| x >= 0.0)
            && offset_from_minimum.into_iter().all(|x| x >= 0.0);

        is_in
    }

    pub fn encloses_point_and_children<V: ExtendedVertex<N>>(
        &self,
        bounding_box: &BoundingBox<V, N>,
        point: Vector<N>,
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

    fn overlaps_bounding_box_and_children<'a, 'b, V: ExtendedVertex<N>>(
        &'a self,
        self_boundingbox: &'a BoundingBox<V, N>,
        other: &'b Self,
        other_boundingbox: &'b BoundingBox<V, N>,
    ) -> Option<impl Iterator<Item = (&'a BoundingBoxElement<N>, &'b BoundingBoxElement<N>)>> {
        let is_overlap = self.overlaps_bounding_box_single(other);
        is_overlap
            .then(|| {
                match (
                    self.get_children(self_boundingbox),
                    other.get_children(other_boundingbox),
                ) {
                    (Some([selfleft, selfright]), Some([otherleft, otherright])) => [
                        selfleft.overlaps_bounding_box_and_children(
                            self_boundingbox,
                            otherleft,
                            other_boundingbox,
                        ),
                        selfleft.overlaps_bounding_box_and_children(
                            self_boundingbox,
                            otherright,
                            other_boundingbox,
                        ),
                        selfright.overlaps_bounding_box_and_children(
                            self_boundingbox,
                            otherleft,
                            other_boundingbox,
                        ),
                        selfright.overlaps_bounding_box_and_children(
                            self_boundingbox,
                            otherright,
                            other_boundingbox,
                        ),
                    ]
                    .into_iter()
                    .flatten()
                    .flatten()
                    .collect(),
                    (None, Some([otherleft, otherright])) => [
                        self.overlaps_bounding_box_and_children(
                            self_boundingbox,
                            otherleft,
                            other_boundingbox,
                        ),
                        self.overlaps_bounding_box_and_children(
                            self_boundingbox,
                            otherright,
                            other_boundingbox,
                        ),
                    ]
                    .into_iter()
                    .flatten()
                    .flatten()
                    .collect(),
                    (Some([selfleft, selfright]), None) => [
                        selfleft.overlaps_bounding_box_and_children(
                            self_boundingbox,
                            other,
                            other_boundingbox,
                        ),
                        selfright.overlaps_bounding_box_and_children(
                            self_boundingbox,
                            other,
                            other_boundingbox,
                        ),
                    ]
                    .into_iter()
                    .flatten()
                    .flatten()
                    .collect(),
                    (None, None) => {
                        println!("leaf!");
                        vec![(self, other)]
                    }
                }
            })
            .filter(|vec| !vec.is_empty())
            .map(IntoIterator::into_iter)
    }

    pub fn get_triangles<'a, V: ExtendedVertex<N>>(
        &self,
        bounding_box: &'a BoundingBox<V, N>,
    ) -> &'a [[V; 3]] {
        &bounding_box.triangles[self.lower_bound_index..self.upper_bound_index]
    }

    fn get_children<'a, V: ExtendedVertex<N>>(
        &self,
        bounding_box: &'a BoundingBox<V, N>,
    ) -> Option<[&'a BoundingBoxElement<N>; 2]> {
        self.children
            .map(|arr_children| arr_children.map(|index| bounding_box.get(index)))
    }

    pub fn overlaps_bounding_box_single(&self, other: &Self) -> bool {
        let lines: [_; N] = array::from_fn(|index| {
            let one_lb = self.lower_bounds[index];
            let one_ub = self.upper_bounds[index];
            let two_lb = other.lower_bounds[index];
            let two_ub = other.upper_bounds[index];

            TestBoxVertex {
                one_lb,
                one_ub,
                two_lb,
                two_ub,
            }
        });

        lines.into_iter().all(line_overlap)
    }
}

impl<V: CubicVertex> BoundingBox<V, 3> {
    pub fn transform_by_pose(&self, pose: Pose) -> Self {
        let inner = self
            .inner
            .iter()
            .map(|inner| {
                let lower_bounds = pose.act_on(inner.lower_bounds);
                let upper_bounds = pose.act_on(inner.upper_bounds);

                BoundingBoxElement {
                    upper_bounds,
                    lower_bounds,
                    ..inner.clone()
                }
            })
            .collect();

        let triangles = self
            .triangles
            .iter()
            .map(|triangle| triangle.map(|vertex| vertex.transform_by_pose(pose)))
            .collect();

        Self { triangles, inner }
    }

    pub fn detect_collision<const N: usize>(
        &self,
        self_pose: Pose,
        other: &Self,
        other_pose: Pose,
    ) -> Option<ArrayVec<(Vec<[V; 3]>, Vec<[V; 3]>), N>> {
        let self_transformed = self.transform_by_pose(self_pose);
        let other_transformed = other.transform_by_pose(other_pose);

        self_transformed
            .get(0)
            .overlaps_bounding_box_and_children(
                &self_transformed,
                other_transformed.get(0),
                &other_transformed,
            )
            .map(Iterator::collect::<ArrayVec<_, _>>)
            .map(|vec| {
                vec.map(|(lhs, rhs)| {
                    (
                        lhs.get_triangles(&self_transformed).to_vec(),
                        rhs.get_triangles(&other_transformed).to_vec(),
                    )
                })
            })
    }
}

impl BoundingBox<SimpleVertex, 3> {
    pub fn from_cubic(model: &Cubic, depth: usize) -> Option<Self> {
        let triangles = model.triangles();
        Self::new_from_triangles(triangles, depth)
    }

    pub fn make_model(&self, level: usize, material: Rc<Material>) -> super::cubic::Builder {
        let triangles = self.get(0).make_model(self, level);
        let mut mesh_builder = Mesh::builder().material(material);
        mesh_builder.extend_triangle(triangles);
        let mut cubic_builder = Cubic::builder();
        cubic_builder.push_mesh(mesh_builder);
        cubic_builder
    }
}

impl BoundingBoxElement<3> {
    fn make_cube(&self) -> impl Iterator<Item = [IncompleteVertex; 3]> {
        vertex_array_cube(self.lower_bounds, self.upper_bounds)
    }

    pub fn make_model<'a, V: CubicVertex>(
        &'a self,
        bounding_box: &'a BoundingBox<V, 3>,
        level: usize,
    ) -> Box<dyn Iterator<Item = [IncompleteVertex; 3]> + 'a> {
        if let Some([lhs, rhs]) = self.get_children(bounding_box)
            && level != 0
        {
            let iter = [
                lhs.make_model(bounding_box, level - 1),
                rhs.make_model(bounding_box, level - 1),
            ]
            .into_iter()
            .flatten();
            Box::new(iter)
            //.collect()
        } else {
            let triangles = self.make_cube();
            Box::new(triangles)
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

fn line_overlap(input: TestBoxVertex) -> bool {
    let TestBoxVertex {
        one_lb,
        one_ub,
        two_lb,
        two_ub,
    } = input;
    let fail = (one_ub < two_lb) || (two_ub < one_lb);
    !fail
}

mod test {
    use graphics::linear_algebra::Vector;

    use crate::modelling::bounding_box::{BoundingBoxElement, TestBoxVertex, line_overlap};

    #[test]
    fn test_point_in_square() {
        use crate::linear_algebra::Vector;
        use crate::modelling::BoundingBox;

        let coords = [
            Vector::new_zero(),
            Vector::new([0.0, 1.0]),
            Vector::new([1.0, 1.0]),
            Vector::new([1.0, 0.0]),
        ];

        let triangles = [[0, 1, 2], [0, 2, 3]];

        let bound = BoundingBox::new_from_vec_indexes(coords.to_vec(), &triangles, 0).unwrap();

        println!("{bound:#?}");

        let good_point = Vector::new([0.5, 0.5]);
        let good_point2 = Vector::new([1.0, 0.5]);
        let good_point3 = Vector::new([1.0, 1.0]);
        let bad_point = Vector::new([2.0, 0.0]);
        let bad_point2 = Vector::new([2.0, 2.0]);
        let bad_point3 = Vector::new([-2.0, -2.0]);

        assert!(bound.encloses_point(good_point));
        assert!(bound.encloses_point(good_point2));
        assert!(bound.encloses_point(good_point3));
        assert!(!bound.encloses_point(bad_point3));
        assert!(!bound.encloses_point(bad_point2));
        assert!(!bound.encloses_point(bad_point));
    }

    #[test]
    fn test_line_overlap() {
        let input = TestBoxVertex {
            one_lb: -1.0,
            one_ub: 1.0,
            two_lb: 0.5,
            two_ub: 2.0,
        };

        assert!(line_overlap(input))
    }

    #[test]
    fn test_overlap() {
        let bounding_box = BoundingBoxElement {
            upper_bounds: Vector::new([1.0, 1.0]),
            lower_bounds: Vector::new([-1.0, -1.0]),
            children: None,
            lower_bound_index: 0,
            upper_bound_index: 0,
        };

        let other_box = BoundingBoxElement {
            upper_bounds: Vector::new([2.0, 0.5]),
            lower_bounds: Vector::new([-2.0, -0.5]),
            children: None,
            lower_bound_index: 0,
            upper_bound_index: 0,
        };

        assert!(bounding_box.overlaps_bounding_box_single(&other_box));
    }
}
