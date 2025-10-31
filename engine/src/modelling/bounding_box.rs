use std::fmt::Debug;

use graphics::ActiveShaderProgram;
use graphics::colour::ColourRGBA;
use graphics::framebuffer::ActiveFramebuffer;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::linear_algebra::Vector;

mod element;
pub use element::BoundingBoxElement;
use graphics::vertex::Vertex;

mod active_element;

mod group;
pub use group::Group as BoundingBoxGroup;

mod skeleton_bb;
pub use skeleton_bb::{SkeletonBb, SkeletonBbGroup};

use crate::modelling::advanced_vertex::ExtendedVertex;
use crate::modelling::cubic::geometry::Pose;
use crate::modelling::{Cubic, CubicVertex, SimpleVertex, Skeleton, Volume};
use crate::{EngResult, TransformBy};

#[derive(Debug)]
pub struct BoundingBox<V: ExtendedVertex<DIMS>, const DIMS: usize, T>
where
    Vector<DIMS>: Volume<DIMS>,
    [(); Vector::<DIMS>::ELEMENT_COUNT]:,
{
    inner: Vec<BoundingBoxElement<V, DIMS, T>>,
}

impl<V: ExtendedVertex<DIMS>, const DIMS: usize, T> BoundingBox<V, DIMS, T>
where
    Vector<DIMS>: Volume<DIMS>,
    [(); Vector::<DIMS>::ELEMENT_COUNT]:,
{
    fn get(&self, index: usize) -> &BoundingBoxElement<V, DIMS, T> {
        &self.inner[index]
    }

    pub(crate) fn new_from_vec_indexes<F: FnMut(&[(Vector<DIMS>, [V; 3])]) -> T>(
        vectors: Vec<V>,
        triangles: &[[usize; 3]],
        depth: usize,
        init: F,
    ) -> Option<Self> {
        let new_triangles: Option<Vec<_>> = triangles
            .into_iter()
            .map(|arr| arr.map(|index| vectors.get(index).copied()).transpose())
            .collect();
        let new_triangles = new_triangles?;
        Self::new_from_triangles(new_triangles, depth, init)
    }

    pub(crate) fn new_from_triangles<F: FnMut(&[(Vector<DIMS>, [V; 3])]) -> T>(
        triangles: Vec<[V; 3]>,
        depth: usize,
        mut init: F,
    ) -> Option<Self> {
        let mut triangles_with_mid: Vec<_> = triangles.iter().map(
            |&triangle| {
                let vertices = triangle.map(|vertex| vertex.position().truncate(/* currently required to force the vectors to the correct length */));
                let mut mid: Vector<_> = vertices.into_iter().sum();
                const ONE_THIRD: f32 = 1.0/3.0;
                mid = mid.scale(ONE_THIRD);

                (mid, triangle)
            }
        ).collect();

        let mut accumulator = Vec::new();

        BoundingBoxElement::new_from_many_inner(
            &mut accumulator,
            &mut triangles_with_mid,
            depth,
            &mut init,
        )?;

        Some(BoundingBox { inner: accumulator })
    }

    pub fn encloses_point(&self, point: Vector<DIMS>) -> bool {
        self.get(0).encloses_point_and_children(self, point)
    }

    pub(crate) fn draw<D: FramebufferWithDepth<1>, F: Fn(&T) -> ColourRGBA>(
        &self,
        level: usize,
        pose: Pose,
        active_shader: &mut ActiveShaderProgram<'_, '_, '_, Vector<DIMS>, (), D::Tex, 1>,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, 1, D>,
        colour: F,
    ) -> EngResult<(), !> {
        active_shader.set_uniform("model", &pose.as_matrix());
        self.get(0)
            .draw_recursive(self, level, active_shader, active_framebuffer, &colour)
    }
}

impl<V: CubicVertex, T> BoundingBox<V, 3, T> {
    pub fn detect_collision<F: Fn(&mut T), O: Fn(&mut T)>(
        lhs: &Self,
        lhs_pose: Pose,
        lhs_alter_meta: F,
        rhs: &Self,
        rhs_pose: Pose,
        rhs_alter_meta: O,
    ) -> Vec<(Vec<[V; 3]>, Vec<[V; 3]>)> {
        BoundingBoxElement::overlaps_bounding_box_and_children(
            lhs.get(0),
            lhs,
            &lhs_pose,
            &lhs_alter_meta,
            rhs.get(0),
            rhs,
            &rhs_pose,
            0,
            &rhs_alter_meta,
        )
        .map(|(lhs, rhs)| {
            (
                lhs.get_triangles()
                    .into_iter()
                    .map(|triangle| triangle.transform_by(&lhs_pose))
                    //.copied()
                    .collect(),
                rhs.get_triangles()
                    .into_iter()
                    .map(|triangle| triangle.transform_by(&rhs_pose))
                    //.copied()
                    .collect(),
            )
        })
        .collect()
    }
}

impl<T> BoundingBox<SimpleVertex, 3, T> {
    pub fn from_cubic<F: FnMut(&[(Vector<3>, [SimpleVertex; 3])]) -> T>(
        model: &Cubic,
        depth: usize,
        init: F,
    ) -> Option<Self> {
        let triangles = model
            .triangles()
            .into_iter()
            .map(|(_, triangle)| triangle)
            .collect();
        Self::new_from_triangles(triangles, depth, init)
    }
}

mod test {
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

        let bound =
            BoundingBox::new_from_vec_indexes(coords.to_vec(), &triangles, 0, |_| ()).unwrap();

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
        use crate::transform::NoTransform;

        let bounding_box = BoundingBoxElement::<Vector<2>, _, ()> {
            upper_bound: Vector::new([1.0, 1.0]),
            lower_bound: Vector::new([-1.0, -1.0]),
            ..Default::default()
        };

        let other_box = BoundingBoxElement::<Vector<2>, _, ()> {
            upper_bound: Vector::new([2.0, 0.5]),
            lower_bound: Vector::new([-2.0, -0.5]),
            ..Default::default()
        };

        assert!(bounding_box.overlaps_bounding_box_single(&NoTransform, &other_box, &NoTransform));
    }
}
