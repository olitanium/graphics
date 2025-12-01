use crate::modelling::cubic::geometry::Pose;
use crate::modelling::cubic::skeleton::SkeletonBoneIndex;

mod builder;
pub use builder::Builder as BoneBuilder;

#[derive(Debug, Clone)]
pub struct Bone {
    pub(crate) parent: SkeletonBoneIndex,
    inverse_bind_pose: Pose,
}

impl Bone {
    pub fn builder(parent: SkeletonBoneIndex) -> BoneBuilder {
        BoneBuilder::new(parent)
    }

    pub const fn inverse_bind_pose(&self) -> Pose {
        self.inverse_bind_pose
    }
}
