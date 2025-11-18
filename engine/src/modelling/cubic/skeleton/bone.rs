use std::cell::Cell;
use std::num::NonZeroUsize;

use crate::modelling::Skeleton;
use crate::modelling::cubic::geometry::{Pose, SkeletalAnimation};
use crate::transform::Transform;

mod builder;
pub use builder::Builder;

#[derive(Debug, Clone)]
pub struct Bone {
    pub(crate) parent: usize,
    pub(crate) inverse_bind_pose: Pose,
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

    // pub fn set(&mut self, pose: Pose) {
    // let mut animation = BoneAnimation::new();
    // animation
    // .push_final_non_repeat(pose)
    // .expect("Fresh animation above");
    //
    // self.animations = Rc::new([animation]);
    // }
}

impl Bone {
    pub fn get_pose(
        &self,
        bone_index: NonZeroUsize,
        root: Pose,
        skeleton: &Skeleton,
        animation: &SkeletalAnimation,
        time: f32,
    ) -> Option<Pose> {
        // TODO: caching
        // if let Some(pose) = self
        // .last_query
        // .get()
        // .and_then(|(inner_hint, pose)| (inner_hint == (animation,
        // time)).then_some(pose)) {
        // return pose;
        // }

        // Fails is Parent is not a valid bone or if the parent bone could not get a
        // pose. Fatal.
        let parent_pose =
            skeleton.get_bone_pose(root, false, self.parent, Some(animation), time)?;

        // Fails if this bone does not exist in the animation. Fatal.
        let own_pose = animation.get_pose(bone_index, time)?;

        let total_pose = parent_pose.transform(&own_pose);

        // TODO: caching
        // self.last_query.set(Some(((animation, time), total_pose)));

        Some(total_pose)
    }
}
