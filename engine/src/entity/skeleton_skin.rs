use graphics::linear_algebra::Vector;
use graphics::vertex::Vertex;

use crate::modelling::cubic::SkeletonCubic;
use crate::modelling::cubic::geometry::Pose;
use crate::modelling::{SimpleCubic, SimpleVertex, SkeletalVertex, SkeletonBb, Volume};

#[derive(Debug)]
pub struct SkinningEntity<T> {
    pub model: SkeletonCubic,
    pub pose: Pose,
    pub mass: f32,
    bounding_box: Option<SkeletonBb<SkeletalVertex, T>>,
    pub velocity: Vector<3>,
}

impl<T: Default> Default for SkinningEntity<T> {
    fn default() -> Self {
        Self {
            mass: f32::INFINITY,
            pose: Pose::IDENTITY,
            model: Default::default(),
            bounding_box: Default::default(),
            velocity: Default::default(),
        }
    }
}

// TODO: more generic, let the user give init
impl SkinningEntity<()> {
    pub fn from_skeletoncubic(model: SkeletonCubic, depth: usize, mass: f32) -> Option<Self> {
        let bounding_box = SkeletonBb::from_skeletoncubic(&model, depth, |_| ());

        Some(Self {
            model,
            mass,
            bounding_box,
            ..Self::default()
        })
    }
}

impl<T> SkinningEntity<T> {
    pub fn bounding_box(&self) -> Option<&SkeletonBb<SkeletalVertex, T>>
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
