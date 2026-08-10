use geometry::Pose;
use crate::modelling::cubic::skeleton::SkeletonBoneIndex;

mod builder;
pub use builder::Builder as BoneBuilder;

#[derive(Debug)]
pub struct Bone {
    pub(crate) parent: SkeletonBoneIndex,
    inverse_bind_pose: Pose,
}

impl Clone for Bone {
    fn clone(&self) -> Self {
        Self {
            parent: self.parent.clone_inner(),
            inverse_bind_pose: self.inverse_bind_pose,
        }
    }
}

impl Bone {
    pub fn builder(parent: &SkeletonBoneIndex) -> BoneBuilder<'_> {
        BoneBuilder::new(parent)
    }

    pub const fn inverse_bind_pose(&self) -> Pose {
        self.inverse_bind_pose
    }
}
