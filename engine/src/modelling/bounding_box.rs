use std::array;
use std::rc::Rc;

use graphics::colour::ColourRGBA;
use graphics::linear_algebra::Vector;
use graphics::texture::FlatTexture;
use graphics::vertex::Vertex;

use crate::modelling::Cubic;
use crate::modelling::cubic::geometry::Pose;
use crate::modelling::cubic::material::Material;
use crate::modelling::test_models::vertex_array_cube;

#[derive(Debug, Clone)]
pub struct BoundingBox<const N: usize> {
    lower_bounds: Vector<N>,
    upper_bounds: Vector<N>,

    children: Option<(Box<Self>, Box<Self>)>,
}

fn make_bound<const N: usize>(
    vectors: &[Vector<N>],
    func: fn(f32, f32) -> f32,
) -> Option<Vector<N>> {
    let bounds =
        array::from_fn(|index| vectors.into_iter().map(|vec| vec[index]).reduce(func)).transpose();
    bounds.map(Vector::new)
}

fn make_upper_bound<const N: usize>(vectors: &[Vector<N>]) -> Option<Vector<N>> {
    make_bound(vectors, f32::max)
}

fn make_lower_bound<const N: usize>(vectors: &[Vector<N>]) -> Option<Vector<N>> {
    make_bound(vectors, f32::min)
}

impl<const N: usize> BoundingBox<N> {
    pub fn blank() -> Self {
        let nan_vec = Vector::new([f32::NAN; N]);
        Self {
            lower_bounds: nan_vec,
            upper_bounds: nan_vec,

            children: None,
        }
    }
}

impl<const N: usize> BoundingBox<N> {
    pub fn new_from_vec_indexes<V: Vertex>(
        vectors: &[V],
        triangles: &[[usize; 3]],
        depth: usize,
    ) -> Option<Self>
    where
        [(); V::DIMENSIONS]:,
    {
        let new_triangles: Option<Vec<_>> = triangles
            .into_iter()
            .map(|arr| arr.map(|index| vectors.get(index).copied()).transpose())
            .collect();
        let new_triangles = new_triangles?;
        Self::new_from_triangles(&new_triangles, depth)
    }

    pub fn new_from_triangles<V: Vertex>(triangles: &[[V; 3]], depth: usize) -> Option<Self>
    where
        [(); V::DIMENSIONS]:,
    {
        let mut triangles_with_mid: Vec<_> = triangles.iter().map(
            |triangle| {
                let vertices = triangle.map(|vertex| vertex.position().truncate(/* currently required to force the vectors to the correct length */));
                let mut mid: Vector<_> = vertices.into_iter().sum();
                mid = mid.scale(1.0/3.0);

                (mid, vertices)
            }
        ).collect();

        Self::new_from_many_inner(&mut triangles_with_mid, depth)
    }

    fn new_from_many_inner(
        triangles: &mut [(Vector<N>, [Vector<N>; 3])],
        depth: usize,
    ) -> Option<Self> {
        // partition is done by the centres of the triangles, triangles are by triplets
        // from the element array buffer. get all the vertices and all of the
        // relevant triangles (as triplets of indices) build all the triangles
        // and get their centres. (this can be cached along with the element array
        // buffer triplets) extend based on accessing the vertices array at the
        // given positions for each vertex for each triangle. partition based on
        // the (cacehed) centres for each triangle.

        let vectors: Vec<_> = triangles.into_iter().flat_map(|(_, arr)| *arr).collect();
        let upper_bounds = make_upper_bound(&vectors)?;
        let lower_bounds = make_lower_bound(&vectors)?;

        // vectors must be populated
        let other = if depth != 0 {
            let dimensions = upper_bounds - lower_bounds;
            let (max_index, _) = dimensions
                .into_iter()
                .enumerate()
                .max_by(|(_, size), (_, size2)| {
                    size.partial_cmp(size2).expect(
                        "vectors has values, so has filled the previous step. No NAN in this \
                         vector",
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
                .partition_point(|(vector, _)| vector[max_index] <= centre_point[max_index]);

            let (lower, upper) = triangles.split_at_mut(partition);

            let lower_bounding_box = Self::new_from_many_inner(lower, depth - 1).map(Box::new);
            let upper_bounding_box = Self::new_from_many_inner(upper, depth - 1).map(Box::new);

            Option::zip(lower_bounding_box, upper_bounding_box)
        } else {
            None
        };

        Some(Self {
            upper_bounds,
            lower_bounds,
            children: other,
        })
    }
}

impl<const N: usize> BoundingBox<N> {
    pub fn encloses_point_single(&self, point: Vector<N>) -> bool {
        let offset_from_minimum = point - self.lower_bounds; // if in, all positive
        let offset_from_maximum = self.upper_bounds - point; // if in, all positive

        let is_in = offset_from_maximum
            .into_inner()
            .into_iter()
            .all(|x| x >= 0.0)
            && offset_from_minimum
                .into_inner()
                .into_iter()
                .all(|x| x >= 0.0);

        is_in
    }

    pub fn encloses_point(&self, point: Vector<N>) -> bool {
        let is_in = self.encloses_point_single(point);

        match &self.children {
            Some((left, right)) => {
                is_in && (left.encloses_point(point) || right.encloses_point(point))
            }
            None => is_in,
        }
    }

    fn overlaps_bounding_box_single(&self, other: &Self) -> bool {
        // test intersection between this bounding box and the other. If pass, then
        // recursively test each child with each otehr child. check if any eight
        // points of one box are within the other box;
        let mut iter = (0..usize::pow(2, N as u32)).map(|bin| {
            let arr: [_; N] = array::from_fn(|index| {
                let is_one = (bin & (1 << index)) != 0;
                if is_one {
                    self.lower_bounds[index]
                } else {
                    self.upper_bounds[index]
                }
            });

            Vector::new(arr)
        });

        iter.any(|point| other.encloses_point_single(point))
    }

    fn overlaps_bounding_box(&self, other: &Self) -> bool {
        let is_overlap = self.overlaps_bounding_box_single(other);

        match (&self.children, &other.children) {
            (Some((selfleft, selfright)), Some((otherleft, otherright))) => {
                is_overlap
                    && (selfleft.overlaps_bounding_box(otherleft)
                        || selfleft.overlaps_bounding_box(otherright)
                        || selfright.overlaps_bounding_box(otherleft)
                        || selfright.overlaps_bounding_box(otherright))
            }
            (None, Some((otherleft, otherright))) => {
                is_overlap
                    && (self.overlaps_bounding_box(otherleft)
                        || self.overlaps_bounding_box(otherright))
            }
            (Some((selfleft, selfright)), None) => {
                is_overlap
                    && (selfleft.overlaps_bounding_box(other)
                        || selfright.overlaps_bounding_box(other))
            }
            (None, None) => is_overlap,
        }
    }
}

impl BoundingBox<3> {
    fn transform_by_pose(&self, pose: Pose) -> Self {
        let lower_bounds = pose.act_on(self.lower_bounds);
        let upper_bounds = pose.act_on(self.upper_bounds);

        let children = self.children.as_ref().map(|(lhs, rhs)| {
            (
                Box::new(lhs.transform_by_pose(pose)),
                Box::new(rhs.transform_by_pose(pose)),
            )
        });

        Self {
            lower_bounds,
            upper_bounds,

            children,
        }
    }

    pub fn detect_collision(&self, self_pose: Pose, other: &Self, other_pose: Pose) -> bool {
        let transform_other = other_pose.apply_after(self_pose.inverse());

        let transformed = other.transform_by_pose(transform_other);

        self.overlaps_bounding_box(&transformed)
    }

    fn make_model_this(&self) -> Cubic {
        Cubic::builder()
            .push_mesh_from(
                Rc::new(vertex_array_cube(self.lower_bounds, self.upper_bounds)),
                Rc::new(
                    Material::builder()
                        .diffuse(FlatTexture::monochrome(ColourRGBA::new([
                            1.0, 0.0, 0.0, 0.5,
                        ])))
                        .build(),
                ),
                0,
            )
            .build()
    }

    pub fn make_model(&self, level: usize) -> Vec<Cubic> {
        if let Some((lhs, rhs)) = self.children.as_ref()
            && level != 0
        {
            [lhs.make_model(level - 1), rhs.make_model(level - 1)]
                .into_iter()
                .flatten()
                .collect()
        } else {
            vec![self.make_model_this()]
        }
    }

    pub fn from_cubic(model: &Cubic, depth: usize) -> Option<Self> {
        let triangles = model.triangles();
        Self::new_from_triangles(&triangles, depth)
    }
}

mod test {
    #[test]
    fn test1() {
        use std::array;

        use graphics::linear_algebra::Vector;

        const N: usize = 3;
        let iter = (0..usize::pow(2, N as u32)).map(|bin| {
            let arr: [_; N] = array::from_fn(|index| {
                let is_one = (bin & (1 << index)) != 0;
                if is_one { 1.0 } else { 0.0 }
            });

            Vector::new(arr)
        });

        let vec: Vec<_> = iter.collect();
        assert_eq!(vec.len(), 8);
    }

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

        let bound = BoundingBox::new_from_vec_indexes(&coords, &triangles, 0).unwrap();

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
}
