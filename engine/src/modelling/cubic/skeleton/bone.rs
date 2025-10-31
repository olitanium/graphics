use std::cell::Cell;
use std::rc::Rc;

use crate::modelling::Skeleton;
use crate::modelling::cubic::geometry::{Animation, Pose};
use crate::transform::Transform;

mod builder;
pub use builder::Builder;

#[derive(Debug, Clone)]
pub struct Bone {
    pub(crate) parent: usize,
    pub(crate) inverse_bind_pose: Pose,
    pub(crate) animations: Rc<[Animation]>,
    pub(crate) last_query: Cell<Option<((usize /* Animation index */, f32 /* time */), Pose)>>,
}

impl Bone {
    pub fn builder(parent: usize) -> Builder {
        Builder::new(parent)
    }

    // pub(crate) fn root() -> Self {
    // Self {
    // parent: None,
    // inverse_bind_pose: Default::default(),
    // animation: Default::default(),
    // last_query: Default::default(),
    // }
    // }

    pub fn inverse_bind_pose(&self) -> Pose {
        self.inverse_bind_pose
    }

    pub fn set(&mut self, pose: Pose) {
        let mut animation = Animation::new();
        animation
            .push_final_non_repeat(pose)
            .expect("Fresh animation above");

        self.animations = Rc::new([animation]);
    }
}

impl Bone {
    pub fn get_pose(&self, skeleton: &Skeleton, animation: usize, time: f32) -> Pose {
        if let Some(pose) = self
            .last_query
            .get()
            .and_then(|(inner_hint, pose)| (inner_hint == (animation, time)).then_some(pose))
        {
            return pose;
        }

        let parent_pose = skeleton.get_bone_pose(false, self.parent, animation, time);

        let own_pose = self
            .animations
            .get(animation)
            .map(|anim| anim.get_pose(time))
            .unwrap_or_default();

        let total_pose = parent_pose.transform(&own_pose);

        self.last_query.set(Some(((animation, time), total_pose)));

        total_pose
    }
}
