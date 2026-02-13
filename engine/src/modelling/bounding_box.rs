use std::fmt::Debug;
use std::sync::LazyLock;

use graphics::colour::ColourRGBA;
use graphics::framebuffer::ActiveFramebuffer;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::linear_algebra::Vector;
use graphics::texture::FlatTexture;
use graphics::vertex_array::VertexArray;
use graphics::{ActiveShaderProgram, Model, ShaderProgram};

mod element;
pub use element::BoundingBoxElement;
use graphics::vertex::Vertex;

mod active_element;

mod group;
pub use group::Group as BoundingBoxGroup;

mod skeleton_bb;
pub use skeleton_bb::{DrawError as SkeletonBbDrawError, SkeletonBb, SkeletonBbGroup};

pub mod collision;
pub use collision::Collision;

use crate::modelling::advanced_vertex::ExtendedVertex;
use crate::modelling::bounding_box::element::DrawRecursiveError;
use crate::modelling::cubic::geometry::Pose;
use crate::modelling::{CubicVertex, SimpleVertex, SkeletonCubic, Volume};
use crate::transform::Transform;

#[derive(Debug, Clone)]
pub struct BoundingBox<V: ExtendedVertex<DIMS>, const DIMS: usize, T>
where
    Vector<DIMS>: ExtendedVertex<DIMS>,
    [(); Vector::<DIMS>::ELEMENT_COUNT]:,
{
    inner: Vec<BoundingBoxElement<V, DIMS, T>>,
}

impl<V: ExtendedVertex<DIMS>, const DIMS: usize, T> BoundingBox<V, DIMS, T>
where
    Vector<DIMS>: ExtendedVertex<DIMS>,
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
            .iter()
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
        let mut triangles_with_mid: Vec<_> = triangles
            .iter()
            .map(|&triangle| {
                let vertices = triangle.map(|vertex| vertex.position());
                let mut mid: Vector<_> = vertices.into_iter().sum();
                const ONE_THIRD: f32 = 1.0 / 3.0;
                mid = mid.scale(ONE_THIRD);

                (mid, triangle)
            })
            .collect();

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

    pub fn detect_collision<
        V2: ExtendedVertex<DIMS>,
        U,
        F: FnMut(&mut T, &mut U),
        P: Transform<Vector<DIMS>>,
    >(
        lhs: &Self,
        lhs_pose: P,
        rhs: &BoundingBox<V2, DIMS, U>,
        rhs_pose: P,
        mut alter_meta: F,
    ) -> bool
    where
        P: Transform<[V; 3]>,
        P: Transform<[V2; 3]>,
    {
        BoundingBoxElement::overlaps_bounding_box_and_children(
            lhs.get(0),
            lhs,
            &lhs_pose,
            rhs.get(0),
            rhs,
            &rhs_pose,
            &mut alter_meta,
        )
        // .map(|(lhs, rhs)| {
        // (
        // lhs.get_triangles()
        // .iter()
        // .map(|triangle| lhs_pose.transform(triangle))
        // .copied()
        // .collect(),
        // rhs.get_triangles()
        // .iter()
        // .map(|triangle| rhs_pose.transform(triangle))
        // .copied()
        // .collect(),
        // )
        // })
        // .collect()
    }
}

impl<V: ExtendedVertex<DIMS>, const DIMS: usize, T> BoundingBox<V, DIMS, T>
where
    BoundingBoxElement<V, DIMS, T>: BoundingBoxShaderProgram<DIMS>,
    Vector<DIMS>: Volume<DIMS>,
    [(); Vector::<DIMS>::ELEMENT_COUNT]:,
{
    pub(crate) fn draw<
        'a,
        P: Transform<Vector<DIMS>>,
        D: FramebufferWithDepth<1, Tex = FlatTexture>,
        F: Fn(&T) -> ColourRGBA,
    >(
        &self,
        level: usize,
        pose: P,
        active_shader: &mut ActiveShaderProgram<
            'a,
            '_,
            '_,
            VertexArray<Vector<DIMS>>,
            (),
            D::Tex,
            1,
        >,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, 1, D>,
        colour: F,
    ) -> Result<(), DrawRecursiveError<'a>> {
        self.get(0).draw_recursive(
            self,
            &pose,
            level,
            active_shader,
            active_framebuffer,
            &colour,
        )
    }
}

impl<T> BoundingBox<SimpleVertex, 3, T> {
    pub fn from_cubic<F: FnMut(&[(Vector<3>, [SimpleVertex; 3])]) -> T>(
        model: &SkeletonCubic,
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

impl<VThis: CubicVertex, TThis> Collision for BoundingBox<VThis, 3, TThis> {
    type FuncInputs = ();
    type Meta = TThis;
    type Transform<'a> = Pose;

    fn collides_with_primitive<
        V: CubicVertex,
        U,
        F: FnMut(Self::FuncInputs, &mut Self::Meta, &mut U),
    >(
        &self,
        transform: &Self::Transform<'_>,
        target: &BoundingBox<V, 3, U>,
        target_transform: &Pose,
        mut alter_meta: F,
    ) -> bool {
        Self::detect_collision(self, *transform, target, *target_transform, |x, y| {
            alter_meta((), x, y)
        })
    }

    fn collides<
        T: Collision,
        F: FnMut(Self::FuncInputs, &mut Self::Meta, T::FuncInputs, &mut T::Meta),
    >(
        &self,
        self_transform: &Self::Transform<'_>,
        target: &T,
        target_transform: &T::Transform<'_>,
        mut alter_meta: F,
    ) -> bool {
        target.collides_with_primitive(
            target_transform,
            self,
            self_transform,
            |target_input, target_meta, self_meta| {
                alter_meta((), self_meta, target_input, target_meta)
            },
        )
    }
}

// Can I remove or change DIMS?
pub trait BoundingBoxShaderProgram<const DIMS: usize>: Model + Sized
where
    Vector<DIMS>: Vertex,
{
    fn shader_program() -> &'static ShaderProgram<VertexArray<Vector<DIMS>>, (), 1, FlatTexture>;
}

impl<V: ExtendedVertex<3>, T> BoundingBoxShaderProgram<3> for BoundingBoxElement<V, 3, T> {
    fn shader_program() -> &'static ShaderProgram<VertexArray<Vector<3>>, (), 1, FlatTexture> {
        static SHADER_PROGRAM: LazyLock<ShaderProgram<VertexArray<Vector<3>>, (), 1, FlatTexture>> =
            LazyLock::new(|| {
                ShaderProgram::builder()
                    .vertex_shader("shaders/bounding_box/bounding_box.vert")
                    .expect("ERROR")
                    .fragment_shader("shaders/bounding_box/bounding_box.frag")
                    .expect("ERROR")
                    .build()
                    .expect("ERROR")
            });

        &SHADER_PROGRAM
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
