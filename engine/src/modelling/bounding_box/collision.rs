use geometry::Pose;

use crate::modelling::{BoundingBox, CubicVertex};

pub trait Collision {
    type Vertex;
    type Transform<'a>: Clone;
    type Meta;
    type FuncInputs;

    #[doc(hidden)]
    fn collides_with_cubic_primitive<
        'a,
        'b,
        V: CubicVertex,
        T,
        F: FnMut(Self::FuncInputs, &mut Self::Meta, &mut T),
    >(
        &'a self,
        self_transform: Self::Transform<'_>,
        target: &'b BoundingBox<V, 3, T>,
        target_transform: Pose,
        mut alter_meta: F,
    ) -> Option<
        Vec<(
            &'a [Self::Vertex; 3],
            &'b [<BoundingBox<V, 3, T> as Collision>::Vertex; 3],
        )>,
    > {
        self.collides(
            self_transform,
            target,
            target_transform,
            |self_inputs, self_inner, (), bbox_inner| {
                alter_meta(self_inputs, self_inner, bbox_inner)
            },
        )
    }

    fn collides<
        'a,
        'b,
        T: Collision,
        F: FnMut(Self::FuncInputs, &mut Self::Meta, T::FuncInputs, &mut T::Meta),
    >(
        &'a self,
        transform: Self::Transform<'_>,
        target: &'b T,
        target_transform: T::Transform<'_>,
        alter_meta: F,
    ) -> Option<Vec<(&'a [Self::Vertex; 3], &'b [T::Vertex; 3])>>;
}

/// Convenience function to annotate that the bounding box metadata is not
/// altered
#[inline]
pub fn no_alter_meta<A: Collision, B: Collision>(
    _: A::FuncInputs,
    _: &mut A::Meta,
    _: B::FuncInputs,
    _: &mut B::Meta,
) {
}

// TODO: deref collision
// impl<T: Collision, B: Deref<Target = T>> Collision for B {
// type FuncInputs = T::FuncInputs;
// type Meta = T::Meta;
// type Transform<'a> = T::Transform<'a>;
// type Vertex = T::Vertex;
//
// fn collides<
// 'a,
// 'b,
// T2: Collision,
// F: FnMut(Self::FuncInputs, &mut Self::Meta, T2::FuncInputs, &mut T2::Meta),
// >(
// &'a self,
// transform: Self::Transform<'_>,
// target: &'b T2,
// target_transform: T2::Transform<'_>,
// alter_meta: F,
// ) -> Option<Vec<(&'a [Self::Vertex; 3], &'a [T2::Vertex; 3])>>
// where
// for<'c> T2::Transform<'c>: Clone,
// {
// T::collides(self, transform, target, target_transform, alter_meta)
// }
// }
