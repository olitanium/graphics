mod bone;
use std::num::NonZeroUsize;

pub use bone::Bone;
use graphics::linear_algebra::Matrix;

use super::geometry::Pose;
use crate::modelling::cubic::geometry::SkeletalAnimation;
use crate::transform::Transform;

#[derive(Debug, Clone)]
pub struct Skeleton {
    // pub root: Pose,
    // All accesses to `values` is offset by -1, where 0 means accessing the root node above,
    // 1 means accessing values[0], 2 => values[1] etc...
    values: Vec<Bone>,
}

impl Default for Skeleton {
    fn default() -> Self {
        Self::new()
    }
}

impl Skeleton {
    pub const ROOT_BONE: usize = 0;

    pub fn new() -> Self {
        Self {
            // root: Pose::identity(),
            values: vec![],
        }
    }

    /// Returns the index of that bone, or `None` if the parent does not exist.
    pub fn push_bone(&mut self, bone: Bone) -> Option<usize> {
        let (GetBone::RootBone | GetBone::Bone { .. }) = self.get(bone.parent) else {
            return None;
        };

        self.values.push(bone);

        // len is one more than the highest index, but this offset is taken care of in
        // Skeleton::get
        Some(self.values.len())
    }

    pub fn get_all_bones_matrix(
        &self,
        root: Pose,
        animation: &SkeletalAnimation,
        time: f32,
    ) -> Vec<Matrix<4, 4>> {
        self.get_all_bones(root, animation, time)
            .into_iter()
            .map(|pose| pose.as_matrix())
            .collect()
    }

    pub fn get_all_bones(&self, root: Pose, animation: &SkeletalAnimation, time: f32) -> Vec<Pose> {
        (0..)
            .map_while(|index| self.get_bone_pose(root, true, index, animation, time))
            .collect()
    }

    pub(crate) fn get(&self, index: usize) -> GetBone<'_> {
        if let Some(non_zero_index) = NonZeroUsize::new(index) {
            match self.values.get(index - 1) {
                Some(bone) => GetBone::Bone {
                    bone_index: non_zero_index,
                    bone,
                },
                None => GetBone::NoBone,
            }
        } else {
            // i.e. index == 0
            GetBone::RootBone
        }
    }

    pub fn get_bone_pose(
        &self,
        root: Pose,
        relative: bool,
        bone_index: usize,
        animation: &SkeletalAnimation,
        time: f32,
    ) -> Option<Pose> {
        match self.get(bone_index) {
            GetBone::Bone { bone_index, bone } => {
                let pose = bone.get_pose(bone_index, root, self, animation, time)?;

                if relative {
                    Some(pose.transform(&bone.inverse_bind_pose))
                } else {
                    Some(pose)
                }
            }
            GetBone::RootBone => Some(root),
            GetBone::NoBone => None,
        }
    }

    /// Used to determine whether a skeleton and an animation are compatible
    pub(crate) fn signature(&self) -> Vec<usize> {
        self.values.iter().map(|bone| bone.parent).collect()
    }
}

pub enum GetBone<'a> {
    NoBone,
    RootBone,
    Bone {
        bone_index: NonZeroUsize,
        bone: &'a Bone,
    },
}
