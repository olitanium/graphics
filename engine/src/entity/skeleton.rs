use graphics::linear_algebra::Vector;
use graphics::vertex::Vertex;
use utils::partition;

use crate::entity::Mass;
use crate::modelling::cubic::geometry::Pose;
use crate::modelling::{SimpleCubic, SimpleVertex, SkeletonBb, Volume};

#[derive(Debug)]
pub struct SkeletonEntity<T> {
    pub model: SimpleCubic,
    pub pose: Pose,
    pub mass: Mass,
    bounding_box: Option<SkeletonBb<SimpleVertex, T>>,
    pub velocity: Vector<3>,
}

impl<T> Default for SkeletonEntity<T> {
    fn default() -> Self {
        Self {
            mass: Default::default(),
            pose: Default::default(),
            model: Default::default(),
            bounding_box: Default::default(),
            velocity: Default::default(),
        }
    }
}

impl<T> SkeletonEntity<T> {
    pub fn from_cubic<F: FnMut(&[(Vector<3>, [SimpleVertex; 3])]) -> T>(
        model: SimpleCubic,
        depth: usize,
        mass: Mass,
        init: F,
    ) -> Option<Self> {
        let bounding_box = make_bounding_box(&model, depth, init);

        Some(Self {
            model,
            mass,
            bounding_box,
            ..Self::default()
        })
    }
}

pub fn make_bounding_box<T, F: FnMut(&[(Vector<3>, [SimpleVertex; 3])]) -> T>(
    cubic: &SimpleCubic,
    depth: usize,
    init: F,
) -> Option<SkeletonBb<SimpleVertex, T>> {
    let per_mesh = partition(cubic.triangles(), |(bone, _)| vec![bone.0], |(_, tri)| tri);

    SkeletonBb::from_vec_triangle(per_mesh, depth, cubic.skeleton.clone(), init)
}

impl<T> SkeletonEntity<T> {
    pub const fn bounding_box(&self) -> Option<&SkeletonBb<SimpleVertex, T>>
    where
        Vector<3>: Volume<3>,
        [(); Vector::<3>::ELEMENT_COUNT]:,
    {
        self.bounding_box.as_ref()
    }

    pub fn update(&mut self, timestep: f32) {
        let delta_pose = Pose::new_from_translation(self.velocity.scale(timestep));
        self.pose.push_change(delta_pose);
    }
}
