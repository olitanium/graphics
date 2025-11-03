use graphics::linear_algebra::Vector;
use graphics::vertex::Vertex;

use crate::modelling::cubic::geometry::Pose;
use crate::modelling::{Cubic, SimpleVertex, SkeletonBb, Volume};

#[derive(Debug)]
pub struct SkeletonEntity<T> {
    pub model: Cubic,
    pub pose: Pose,
    pub mass: f32,
    bounding_box: Option<SkeletonBb<SimpleVertex, T>>,
    pub velocity: Vector<3>,
}

impl<T: Default> Default for SkeletonEntity<T> {
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
impl SkeletonEntity<(bool, Vec<[SimpleVertex; 3]>)> {
    pub fn from_cubic(model: Cubic, depth: usize, mass: f32) -> Option<Self> {
        let bounding_box = SkeletonBb::from_cubic_simplevertex(&model, depth, |input| {
            (
                false,
                input.into_iter().map(|(_, triangle)| *triangle).collect(),
            )
        });

        Some(Self {
            model,
            mass,
            bounding_box,
            ..Self::default()
        })
    }
}

impl<T> SkeletonEntity<T> {
    pub fn bounding_box(&self) -> Option<&SkeletonBb<SimpleVertex, T>>
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
