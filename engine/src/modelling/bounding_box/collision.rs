use crate::modelling::{BoundingBox, CubicVertex, ExtendedVertex};

pub trait Collision {
    type Vertex: ExtendedVertex;
    type Meta;
    type FuncInputs;
    type Transform<'a>;
    type VertexTransform<'a>;

    #[doc(hidden)]
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
        self.collides(
            self_transform,
            self_vertex_transform,
            target,
            target_transform,
            target_vertex_transform,
            |self_inputs, self_inner, (), bbox_inner| {
                alter_meta(self_inputs, self_inner, bbox_inner)
            },
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
        alter_meta: F,
    ) -> Option<Vec<([Self::Vertex; 3], [T::Vertex; 3])>>;
}

/// Convenience function to annotate that the bounding box metadata is not
/// altered
#[inline]
pub fn no_alter_meta<A: Collision + ?Sized, B: Collision + ?Sized>(
    _: A::FuncInputs,
    _: &mut A::Meta,
    _: B::FuncInputs,
    _: &mut B::Meta,
) -> bool {
    true
}
