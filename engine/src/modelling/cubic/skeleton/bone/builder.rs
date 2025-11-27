use utils::builder;

use super::Bone;
use crate::modelling::{Skeleton, SkeletonBuilder};
use crate::modelling::cubic::geometry::{BoneAnimation, Pose};
use crate::modelling::cubic::skeleton::GetBone;
use crate::transform::Transform;

#[derive(Debug, Clone)]
pub struct Builder {
    parent: usize,
    inverse_bind_pose_relative_to_parent: Option<Pose>,
    animations: Vec<BoneAnimation>,
}

impl Builder {
    builder!(inverse_bind_pose_relative_to_parent: Option<Pose>);

    pub fn new(parent: usize) -> Self {
        Self {
            parent,
            inverse_bind_pose_relative_to_parent: Default::default(),
            animations: vec![],
        }
    }

    pub fn push_animation(mut self, animation: BoneAnimation) -> Self {
        self.animations.push(animation);
        self
    }

    pub fn animations(self, animations: Vec<BoneAnimation>) -> Self {
        Builder { animations, ..self }
    }

    pub fn bind_pose_relative_to_parent(self, bind_pose: Pose) -> Self {
        Self {
            inverse_bind_pose_relative_to_parent: Some(bind_pose.inverse()),
            ..self
        }
    }

    pub fn build(self, skeleton: &SkeletonBuilder) -> Option<Bone> {
        let inverse_bind_pose = match (
            self.inverse_bind_pose_relative_to_parent,
            skeleton.get(self.parent),
        ) {
            (None, GetBone::Bone { bone: parent, .. }) => parent.inverse_bind_pose(),
            (None, GetBone::RootBone) => Pose::default(),
            (_, GetBone::NoBone) => return None,
            (Some(self_inverse), GetBone::Bone { bone: parent, .. }) => {
                parent.inverse_bind_pose().transform(&self_inverse)
            }
            (Some(self_inverse), GetBone::RootBone) => self_inverse,
        };

        Some(Bone {
            parent: self.parent,
            inverse_bind_pose,
        })
    }
}
