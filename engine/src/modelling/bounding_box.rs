use graphics::linear_algebra::Vector;

mod element;
pub use element::BoundingBoxElement;
use graphics::vertex::Vertex;

mod active_element;

use crate::TransformBy;
use crate::modelling::advanced_vertex::ExtendedVertex;
use crate::modelling::cubic::geometry::Pose;
use crate::modelling::{Cubic, CubicVertex, SimpleVertex, Volume};

#[derive(Debug)]
pub struct BoundingBox<V: ExtendedVertex<DIMS>, const DIMS: usize>
where
    Vector<DIMS>: Volume<DIMS>,
    [(); Vector::<DIMS>::ELEMENT_COUNT]:,
{
    triangles: Vec<[V; 3]>,
    inner: Vec<BoundingBoxElement<DIMS>>,
}

impl<V: ExtendedVertex<DIMS>, const DIMS: usize> BoundingBox<V, DIMS>
where
    Vector<DIMS>: Volume<DIMS>,
    [(); Vector::<DIMS>::ELEMENT_COUNT]:,
{
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
    pub fn detect_collision(
        &self,
        self_pose: Pose,
        other: &Self,
        other_pose: Pose,
    ) -> Vec<(Vec<[V; 3]>, Vec<[V; 3]>)> {
        BoundingBoxElement::overlaps_bounding_box_and_children(
            self.get(0),
            self,
            &self_pose,
            other.get(0),
            other,
            &other_pose,
            0,
        )
        .map(|(lhs, rhs)| {
            (
                lhs.get_triangles(&self)
                    .into_iter()
                    .map(|triangle| triangle.transform_by(&self_pose))
                    .collect(),
                rhs.get_triangles(&other)
                    .into_iter()
                    .map(|triangle| triangle.transform_by(&other_pose))
                    .collect(),
            )
        })
        .collect()
    }
}

impl BoundingBox<SimpleVertex, 3> {
    pub fn from_cubic(model: &Cubic, depth: usize) -> Option<Self> {
        let triangles = model
            .triangles()
            .into_iter()
            .map(|(_, triangle)| triangle)
            .collect();
        Self::new_from_triangles(triangles, depth)
    }

    // pub fn make_model(&self, level: usize, _: Rc<Material>) ->
    // super::cubic::Builder { let default_material =
    // Rc::new(Material::black()); let collision_material =
    // Rc::new(Material::from_colour(ColourRGBA::RED)); let mesh_builder =
    // self.get(0)
    // .draw_recursive(self, level, &default_material, &collision_material);
    // let mut mesh_builder = Mesh::builder().material(material);
    // mesh_builder.extend_triangle(triangles);
    // let mut cubic_builder = Cubic::builder();
    // cubic_builder.extend_meshes((0, mesh_builder));
    // cubic_builder
    // }
}

mod test {
    use graphics::vertex_array::VertexArray;

    #[test]
    fn test_point_in_square() {
        use crate::linear_algebra::Vector;
        use crate::modelling::BoundingBox;

        let coords = [
            Vector::ZERO,
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
        use std::cell::Cell;

        use graphics::linear_algebra::Vector;

        use crate::modelling::bounding_box::BoundingBoxElement;
        use crate::transform::NoTransform;

        let bounding_box = BoundingBoxElement {
            upper_bound: Vector::new([1.0, 1.0]),
            lower_bound: Vector::new([-1.0, -1.0]),
            children: None,
            lower_bound_index: 0,
            upper_bound_index: 0,
            vertex_array: VertexArray::empty(),
            texture_choice: 0,
        };

        let other_box = BoundingBoxElement {
            upper_bound: Vector::new([2.0, 0.5]),
            lower_bound: Vector::new([-2.0, -0.5]),
            children: None,
            lower_bound_index: 0,
            upper_bound_index: 0,
            vertex_array: VertexArray::empty(),
            texture_choice: 0,
        };

        assert!(bounding_box.overlaps_bounding_box_single(&NoTransform, &other_box, &NoTransform));
    }
}
