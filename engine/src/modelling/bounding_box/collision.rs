use crate::modelling::cubic::geometry::Pose;
use crate::modelling::{BoundingBox, CubicVertex};

pub trait Collision {
    type Transform<'a>;
    type Inner;
    type FuncInputs;

    fn collides_with_primitive<
        V: CubicVertex,
        T,
        F: FnMut(Self::FuncInputs, &mut Self::Inner),
        O: FnMut(&mut T),
    >(
        &self,
        transform: &Self::Transform<'_>,
        self_alter_meta: F,
        target: &BoundingBox<V, 3, T>,
        target_transform: &Pose,
        mut target_alter_meta: O,
    ) -> bool {
        self.collides(
            transform,
            self_alter_meta,
            target,
            target_transform,
            |(), t| target_alter_meta(t),
        )
    }

    fn collides<
        T: Collision,
        F: FnMut(Self::FuncInputs, &mut Self::Inner),
        O: FnMut(T::FuncInputs, &mut T::Inner),
    >(
        &self,
        transform: &Self::Transform<'_>,
        self_alter_meta: F,
        target: &T,
        target_transform: &T::Transform<'_>,
        target_alter_meta: O,
    ) -> bool;
}
