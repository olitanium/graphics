use core::slice;
use std::array;
use std::rc::Rc;
use std::sync::Mutex;
use std::sync::atomic::AtomicU64;

use graphics::colour::ColourRGBA;
use graphics::linear_algebra::Vector;
use russimp::node;

mod element;
pub use element::BoundingBoxElement;

mod active_element;


use crate::modelling::advanced_vertex::{ExtendedVertex, IncompleteVertex};
use crate::modelling::cubic::{Mesh, MeshBuilder};
use crate::modelling::cubic::geometry::Pose;
use crate::modelling::cubic::material::Material;
use crate::modelling::{Cubic, CubicVertex, SimpleVertex};
use crate::transform::Transform;

#[derive(Debug, Clone)]
pub struct BoundingBox<V: ExtendedVertex<DIMS>, const DIMS: usize> {
    triangles: Vec<[V; 3]>,
    inner: Vec<BoundingBoxElement<DIMS>>,
}

impl<V: ExtendedVertex<DIMS>, const DIMS: usize> BoundingBox<V, DIMS> {
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    fn get(&self, index: usize) -> &BoundingBoxElement<DIMS> {
        &self.inner[index]
    }

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
        let mut triangles_with_mid: Vec<_> = triangles.iter().map(
            |&triangle| {
                let vertices = triangle.map(|vertex| vertex.position().truncate(/* currently required to force the vectors to the correct length */));
                let mut mid: Vector<_> = vertices.into_iter().sum();
                const ONE_THIRD: f32 = 1.0/3.0;
                mid = mid.scale(ONE_THIRD);

                (mid, triangle)
            }
        ).collect();

        let len_triangles = triangles.len();

        let mut so_far = Vec::new();
        BoundingBoxElement::new_from_many_inner(
            &mut so_far,
            &mut triangles_with_mid,
            0,
            len_triangles,
            depth,
        )?;

        Some(BoundingBox {
            triangles,
            inner: so_far,
        })
    }

    pub fn encloses_point(&self, point: Vector<DIMS>) -> bool {
        self.get(0).encloses_point_and_children(self, point)
    }
}


impl<V: CubicVertex> BoundingBox<V, 3> {
    pub fn transform_by_pose(&self, pose: Pose) -> Self {
        let inner = self
            .inner
            .iter()
            .map(|this_elem| {
                let lower_bounds = pose.apply_to(this_elem.lower_bound);
                let upper_bounds = pose.apply_to(this_elem.upper_bound);

                //TODO: sort above and below
                let lower_bounds = pose.transform(&this_elem.lower_bound);
                let upper_bounds = pose.transform(&this_elem.upper_bound);

                BoundingBoxElement {
                    upper_bound: upper_bounds,
                    lower_bound: lower_bounds,
                    children: this_elem.children,
                    lower_bound_index: this_elem.lower_bound_index,
                    upper_bound_index: this_elem.upper_bound_index,
                    collision: this_elem.collision.clone(),
                }
            })
            .collect();

        let triangles = self
            .triangles
            .iter()
            .map(|triangle| triangle.map(|vertex| vertex.transform_by(&pose)))
            .collect();

        Self { triangles, inner }
    }

    pub fn detect_collision(
        &self,
        self_pose: Pose,
        other: &Self,
        other_pose: Pose,
    ) -> Vec<(Vec<[V; 3]>, Vec<[V; 3]>)> {
        //let self_transformed = self.transform_by_pose(self_pose);
        //let other_transformed = other.transform_by_pose(other_pose);

        BoundingBoxElement::overlaps_bounding_box_and_children(
            //self_transformed.get(0),
            //&self_transformed,
            self.get(0),
            self,
            &self_pose,
            //other_transformed.get(0),
            //&other_transformed,
            other.get(0),
            other,
            &other_pose,
            0
        )
        .map(|(lhs, rhs)| {
            (
                lhs.get_triangles(&self).to_vec(),
                rhs.get_triangles(&other).to_vec(),
                //lhs.get_triangles(&self_transformed).to_vec(),
                //rhs.get_triangles(&other_transformed).to_vec(),
            )
        })
        .collect()
    }
}

impl BoundingBox<SimpleVertex, 3> {
    pub fn from_cubic(model: &Cubic, depth: usize) -> Option<Self> {
        let triangles = model.triangles();
        Self::new_from_triangles(triangles, depth)
    }

    pub fn make_model(&self, level: usize, _: Rc<Material>) -> super::cubic::Builder {
        let mesh_builder = self.get(0).make_model(self, level);
        //let mut mesh_builder = Mesh::builder().material(material);
        //mesh_builder.extend_triangle(triangles);
        let mut cubic_builder = Cubic::builder();
        cubic_builder.extend_meshes(mesh_builder);
        cubic_builder
    }
}

mod test {
    use std::cell::Cell;

    use crate::transform::NoTransform;

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
    fn test_overlap() {
        use graphics::linear_algebra::Vector;

        use crate::modelling::bounding_box::BoundingBoxElement;

        let bounding_box = BoundingBoxElement {
            upper_bound: Vector::new([1.0, 1.0]),
            lower_bound: Vector::new([-1.0, -1.0]),
            children: None,
            lower_bound_index: 0,
            upper_bound_index: 0,
            collision: Cell::default(),
        };

        let other_box = BoundingBoxElement {
            upper_bound: Vector::new([2.0, 0.5]),
            lower_bound: Vector::new([-2.0, -0.5]),
            children: None,
            lower_bound_index: 0,
            upper_bound_index: 0,
            collision: Cell::default(),
        };

        assert!(bounding_box.overlaps_bounding_box_single(&NoTransform, &other_box, &NoTransform));
    }
}
