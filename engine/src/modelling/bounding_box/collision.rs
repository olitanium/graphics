use std::ops::Deref;

use crate::modelling::cubic::geometry::Pose;
use crate::modelling::{BoundingBox, CubicVertex};

pub trait Collision {
    type Transform<'a>;
    type Meta;
    type FuncInputs;

    #[doc(hidden)]
    fn collides_with_cubic_primitive<
        V: CubicVertex,
        T,
        F: FnMut(Self::FuncInputs, &mut Self::Meta, &mut T),
    >(
        &self,
        self_transform: Self::Transform<'_>,
        target: &BoundingBox<V, 3, T>,
        target_transform: Pose,
        mut alter_meta: F,
    ) -> bool {
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
        T: Collision,
        F: FnMut(Self::FuncInputs, &mut Self::Meta, T::FuncInputs, &mut T::Meta),
    >(
        &self,
        transform: Self::Transform<'_>,
        target: &T,
        target_transform: T::Transform<'_>,
        alter_meta: F,
    ) -> bool
    where
        for<'a> T::Transform<'a>: Clone;
}

// pub trait TempModelTransform {
// fn get_root_pose(&self) -> Pose;
// }

impl<T: Collision, B: Deref<Target = T>> Collision for B {
    type FuncInputs = T::FuncInputs;
    type Meta = T::Meta;
    type Transform<'a> = T::Transform<'a>;

    fn collides<
        T2: Collision,
        F: FnMut(Self::FuncInputs, &mut Self::Meta, T2::FuncInputs, &mut T2::Meta),
    >(
        &self,
        transform: Self::Transform<'_>,
        target: &T2,
        target_transform: T2::Transform<'_>,
        alter_meta: F,
    ) -> bool
    where
        for<'a> T2::Transform<'a>: Clone,
    {
        T::collides(self, transform, target, target_transform, alter_meta)
    }
}
