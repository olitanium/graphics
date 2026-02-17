use engine::SimpleEntity;
use engine::modelling::Collision;

use crate::properties::Properties;

pub struct Morgan {
    pub inner: SimpleEntity<()>,
}

impl Collision for Morgan {
    type FuncInputs = <SimpleEntity<()> as Collision>::FuncInputs;
    type Meta = <SimpleEntity<()> as Collision>::Meta;
    type Transform<'a> = <SimpleEntity<()> as Collision>::Transform<'a>;

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
        for<'a> T::Transform<'a>: Clone,
    {
        self.inner
            .collides(transform, target, target_transform, alter_meta)
    }
}

impl Properties for Morgan {
    fn is_hookpoint(&self) -> bool {
        false
    }

    fn is_carryable(&self) -> bool {
        true
    }
}
