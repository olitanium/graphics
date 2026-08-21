use std::iter;

use graphics::linear_algebra::Vector;

use crate::entity::physics::Physics;
use crate::entity::{EntityInner, Mass, SimplePhysics, ToAggregateEntity};
use crate::modelling::cubic::SkinningCubic;
use crate::modelling::{Collision, SkeletalVertex, SkeletonBb};

#[derive(Debug, Clone)]
pub struct SkinningEntity<T> {
    pub model: SkinningCubic,
    pub(crate) bounding_box: Option<SkeletonBb<SkeletalVertex, T>>,
    pub physics: SimplePhysics,
}

// TODO: investigate whether this can be derived. There's a problem re: the `T`
impl<T> Default for SkinningEntity<T> {
    fn default() -> Self {
        Self {
            model: Default::default(),
            bounding_box: Default::default(),
            physics: Default::default(),
        }
    }
}

impl<T> SkinningEntity<T> {
    pub fn from_skeletoncubic<F: FnMut(&[(Vector<3>, [SkeletalVertex; 3])]) -> T>(
        model: SkinningCubic,
        depth: usize,
        mass: Mass,
        init: F,
    ) -> Option<Self> {
        let bounding_box = make_bounding_box(&model, depth, init);

        let mut physics = SimplePhysics::default();
        physics.mass = mass;

        Some(Self {
            model,
            bounding_box,
            physics,
            ..Self::default()
        })
    }
}

pub fn make_bounding_box<T, F: FnMut(&[(Vector<3>, [SkeletalVertex; 3])]) -> T>(
    cubic: &SkinningCubic,
    depth: usize,
    init: F,
) -> Option<SkeletonBb<SkeletalVertex, T>> {
    let per_mesh = partition::partition(
        &cubic.triangles(),
        |tri| {
            tri.iter().flat_map(|vertex| {
                iter::zip(vertex.bone_indices, vertex.bone_weights)
                    .filter_map(|(index, weight)| (weight != 0.0).then_some(index as usize))
            })
        },
        |arr| **arr,
    );

    SkeletonBb::from_vec_triangle(per_mesh, depth, init)
}

impl<T> SkinningEntity<T> {
    pub const fn bounding_box(&self) -> Option<&SkeletonBb<SkeletalVertex, T>> {
        self.bounding_box.as_ref()
    }
}

impl<TThis> Collision for SkinningEntity<TThis> {
    type FuncInputs = <SkeletonBb<SkeletalVertex, TThis> as Collision>::FuncInputs;
    type Meta = <SkeletonBb<SkeletalVertex, TThis> as Collision>::Meta;
    type Transform<'a> = <SkeletonBb<SkeletalVertex, TThis> as Collision>::Transform<'a>;
    type Vertex = <SkeletonBb<SkeletalVertex, TThis> as Collision>::Vertex;

    fn collides<
        'a,
        'b,
        T: Collision + ?Sized,
        F: FnMut(Self::FuncInputs, &mut Self::Meta, T::FuncInputs, &mut T::Meta),
    >(
        &'a self,
        transform: Self::Transform<'_>,
        target: &'b T,
        target_transform: T::Transform<'_>,
        alter_meta: F,
    ) -> Option<Vec<([Self::Vertex; 3], [T::Vertex; 3])>> {
        // let Some(bbox) = &self.bounding_box else {
        // return None;
        // };
        self.bounding_box
            .as_ref()?
            .collides(transform, target, target_transform, alter_meta)
    }
}

impl<T> ToAggregateEntity<T, SkinningCubic> for SkinningEntity<T> {
    fn to_pair_model_bbox(
        self,
    ) -> (
        SkinningCubic,
        Option<<SkinningCubic as EntityInner>::BoundingBox<T>>,
    ) {
        let Self {
            model,
            bounding_box,
            ..
        } = self;

        (model, bounding_box)
    }
}

impl<T> Physics for SkinningEntity<T> {
    fn physics(&mut self) -> Option<&mut SimplePhysics> {
        Some(&mut self.physics)
    }
}
