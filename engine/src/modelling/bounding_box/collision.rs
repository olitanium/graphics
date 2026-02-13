use crate::modelling::cubic::geometry::Pose;
use crate::modelling::{BoundingBox, CubicVertex};

pub trait Collision {
    type Transform<'a>;
    type Meta;
    type FuncInputs;

    #[doc(hidden)]
    fn collides_with_primitive<
        V: CubicVertex,
        T,
        F: FnMut(Self::FuncInputs, &mut Self::Meta, &mut T),
    >(
        &self,
        self_transform: &Self::Transform<'_>,
        target: &BoundingBox<V, 3, T>,
        target_transform: &Pose,
        mut alter_meta: F,
    ) -> bool {
        self.collides(
            self_transform,
            target,
            target_transform,
            |self_inputs, self_inner, (), bbox_inner| alter_meta(self_inputs, self_inner, bbox_inner),
        )
    }

    fn collides<
        T: Collision,
        F: FnMut(Self::FuncInputs, &mut Self::Meta, T::FuncInputs, &mut T::Meta),
    >(
        &self,
        transform: &Self::Transform<'_>,
        target: &T,
        target_transform: &T::Transform<'_>,
        alter_meta: F,
    ) -> bool;
}
