use std::fmt::Debug;
use std::sync::LazyLock;

use geometry::{Pose, Transform};
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

mod group;
pub use group::Group as BoundingBoxGroup;

mod skeleton_bb;
pub use skeleton_bb::{
    DrawError as SkeletonBbDrawError,
    MakePreferredTransformFromSkeleton,
    SkeletonBb,
    SkeletonBbGroup,
};

pub mod collision;
pub use collision::{Collision, no_alter_meta};

// pub use crate::modelling::bounding_box::collision::TempModelTransform;
use crate::modelling::bounding_box::element::DrawRecursiveError;
use crate::modelling::extended_vertex::ExtendedVertex;
use crate::modelling::{CubicVertex, SimpleVertex, SkeletonCubic, Volume};

#[derive(Debug, Clone)]
pub struct BoundingBox<V: ExtendedVertex, T> {
    triangles: Vec<(Vector<{ V::LEN }>, [V; 3])>,
    inner: Vec<BoundingBoxElement<V, T>>,
}

impl<V: ExtendedVertex, T> BoundingBox<V, T> {
    fn get(&self, index: usize) -> &BoundingBoxElement<V, T> {
        &self.inner[index]
    }

    fn root_element(&self) -> &BoundingBoxElement<V, T> {
        self.inner
            .last()
            .expect("guaranteed at least one element, or the BB create function would return None")
    }

    pub(crate) fn new_from_vec_indexes<F: FnMut(&[(Vector<{ V::LEN }>, [V; 3])]) -> T>(
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

    pub(crate) fn new_from_triangles<
        I: IntoIterator<Item = [V; 3]>,
        F: FnMut(&[(Vector<{ V::LEN }>, [V; 3])]) -> T,
    >(
        triangles: I,
        depth: usize,
        mut init: F,
    ) -> Option<Self> {
        let mut triangles_with_mid: Vec<_> = triangles
            .into_iter()
            .map(|triangle| {
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
            0,
            depth,
            &mut init,
        )?;

        Some(BoundingBox {
            triangles: triangles_with_mid,
            inner: accumulator,
        })
    }

    pub fn encloses_point(&self, point: Vector<{ V::LEN }>) -> bool {
        self.root_element().encloses_point_and_children(self, point)
    }

    // pub fn temp_make_really_big(self) -> Self {
    // let Self {
    // triangles,
    // mut inner,
    // } = self;
    // let current = mem::replace(
    // &mut inner.first_mut().unwrap().children_or_meta,
    // ChildrenOrMeta::Children([0, 0]),
    // );
    // let ChildrenOrMeta::Meta(refcell) = current else {
    // panic!("no meta")
    // };
    // let new_inner = BoundingBoxElement {
    // lower_bound: vector![-1000.0;{V::LEN}],
    // upper_bound: vector![1000.0;{V::LEN}],
    // children_or_meta: ChildrenOrMeta::Meta(refcell),
    // start_index: 0,
    // len: triangles.len(),
    // triangles: PhantomData,
    // };
    //
    // Self {
    // triangles,
    // inner: vec![new_inner],
    // }
    // }
}

impl<V: ExtendedVertex<LEN = 3>, T> BoundingBox<V, T> {
    fn detect_collision<
        V2: ExtendedVertex<LEN = 3>,
        U,
        F: FnMut(&mut T, &mut U) -> bool,
        P: Transform<Vector<3>>,
        P2: Transform<Vector<3>>,
    >(
        lhs: &Self,
        lhs_pose: &P,
        lhs_vertex_transform: &<V as ExtendedVertex>::PreferredTransform<'_>,
        rhs: &BoundingBox<V2, U>,
        rhs_pose: &P2,
        rhs_vertex_transform: &<V2 as ExtendedVertex>::PreferredTransform<'_>,
        mut alter_meta: F,
    ) -> Option<Vec<([V; 3], [V2; 3])>>
where {
        let mut output = Vec::new();
        BoundingBoxElement::overlaps_bounding_box_and_children(
            lhs.root_element(),
            lhs,
            lhs_pose,
            lhs_vertex_transform,
            rhs.root_element(),
            rhs,
            rhs_pose,
            rhs_vertex_transform,
            &mut alter_meta,
            &mut output,
        );

        if output.is_empty() {
            None
        } else {
            Some(output)
        }
    }
}

// impl<V: ExtendedVertex, T> BoundingBox<V, T> {
// pub fn triangles<'a>(
// &'a self,
// element: &'a BoundingBoxElement<V, T>,
// ) -> &'a [(Vector<{ V::LEN }>, [V; 3])] {
// let lower_bound = element.start_index;
// let upper_bound = element.start_index + element.len;
// &self.triangles[lower_bound..upper_bound]
// }
// }

impl<V: ExtendedVertex, T> BoundingBox<V, T>
where
    BoundingBoxElement<V, T>: BoundingBoxShaderProgram<DIMS = { V::LEN }>,
    Vector<{ <BoundingBoxElement<V, T> as BoundingBoxShaderProgram>::DIMS }>: Vertex,
    Vector<{ V::LEN }>: Volume,
{
    pub(crate) fn draw<
        'a,
        P: Transform<Vector<{ V::LEN }>>,
        D: FramebufferWithDepth<1, Tex = FlatTexture>,
        F: FnMut() -> ColourRGBA,
    >(
        &self,
        level: usize,
        pose: P,
        active_shader: &mut ActiveShaderProgram<
            'a,
            '_,
            '_,
            VertexArray<Vector<{ V::LEN }>>,
            (),
            D::Tex,
            1,
        >,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, 1, D>,
        mut colour: F,
    ) -> Result<(), DrawRecursiveError<'a>> {
        self.root_element().draw_recursive(
            self,
            &pose,
            level,
            active_shader,
            active_framebuffer,
            &mut colour,
        )
    }
}

impl<T> BoundingBox<SimpleVertex, T> {
    pub fn from_cubic<F: FnMut(&[(Vector<3>, [SimpleVertex; 3])]) -> T>(
        model: &SkeletonCubic,
        depth: usize,
        init: F,
    ) -> Option<Self> {
        let triangles = model.triangles().into_iter().map(|(_, triangle)| triangle);
        Self::new_from_triangles(triangles, depth, init)
    }
}

impl<VThis: CubicVertex, TThis> Collision for BoundingBox<VThis, TThis> {
    type FuncInputs = ();
    type Meta = TThis;
    type Transform<'a> = Pose;
    type Vertex = VThis;
    type VertexTransform<'a> = VThis::PreferredTransform<'a>;

    fn collides_with_cubic_primitive<
        V: CubicVertex,
        T,
        F: FnMut(Self::FuncInputs, &mut Self::Meta, &mut T) -> bool,
    >(
        &self,
        self_transform: &Self::Transform<'_>,
        self_vertex_transform: &Self::VertexTransform<'_>,
        target: &BoundingBox<V, T>,
        target_transform: &<BoundingBox<V, T> as Collision>::Transform<'_>,
        target_vertex_transform: &<BoundingBox<V, T> as Collision>::VertexTransform<'_>,
        mut alter_meta: F,
    ) -> Option<
        Vec<(
            [Self::Vertex; 3],
            [<BoundingBox<V, T> as Collision>::Vertex; 3],
        )>,
    > {
        Self::detect_collision(
            self,
            self_transform,
            self_vertex_transform,
            target,
            target_transform,
            target_vertex_transform,
            |x, y| alter_meta((), x, y),
        )
    }

    fn collides<
        T: Collision + ?Sized,
        F: FnMut(Self::FuncInputs, &mut Self::Meta, T::FuncInputs, &mut T::Meta) -> bool,
    >(
        &self,
        self_transform: &Self::Transform<'_>,
        self_vertex_transform: &Self::VertexTransform<'_>,
        target: &T,
        target_transform: &T::Transform<'_>,
        target_vertex_transform: &T::VertexTransform<'_>,
        mut alter_meta: F,
    ) -> Option<Vec<([Self::Vertex; 3], [T::Vertex; 3])>> {
        target
            .collides_with_cubic_primitive(
                target_transform,
                target_vertex_transform,
                self,
                self_transform,
                self_vertex_transform,
                |target_input, target_meta, self_meta| {
                    alter_meta((), self_meta, target_input, target_meta)
                },
            )
            .map(|is_vec| is_vec.into_iter().map(|(a, b)| (b, a)).collect())
    }
}

// Can I remove or change DIMS?
pub trait BoundingBoxShaderProgram: Model + Sized
where
    Vector<{ Self::DIMS }>: Vertex,
{
    type const DIMS: usize;
    fn shader_program()
    -> &'static ShaderProgram<VertexArray<Vector<{ Self::DIMS }>>, (), 1, FlatTexture>;
}

impl<V: ExtendedVertex<LEN = 3>, T: 'static> BoundingBoxShaderProgram for BoundingBoxElement<V, T> {
    type const DIMS: usize = 3;

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
    use graphics::linear_algebra::Vector;

    use crate::modelling::{BoundingBoxElement, ExtendedVertex};

    #[allow(dead_code)]
    fn default<V: ExtendedVertex, T: Default>() -> BoundingBoxElement<V, T> {
        BoundingBoxElement {
            lower_bound: -Vector::X,
            upper_bound: Vector::X,
            children_or_meta: super::element::ChildrenOrMeta::Meta {
                meta: Default::default(),
                triangle_len: 0,
                triangle_start: 0,
            },
        }
    }

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
        use geometry::NoTransform;
        use graphics::linear_algebra::Vector;

        use crate::modelling::bounding_box::BoundingBoxElement;

        let bounding_box = BoundingBoxElement::<Vector<2>, ()> {
            upper_bound: Vector::new([1.0, 1.0]),
            lower_bound: Vector::new([-1.0, -1.0]),
            ..default::<Vector<2>, _>()
        };

        let other_box = BoundingBoxElement::<Vector<2>, ()> {
            upper_bound: Vector::new([2.0, 0.5]),
            lower_bound: Vector::new([-2.0, -0.5]),
            ..default::<Vector<2>, _>()
        };

        assert!(bounding_box.overlaps_bounding_box_single(&NoTransform, &other_box, &NoTransform));
    }
}
