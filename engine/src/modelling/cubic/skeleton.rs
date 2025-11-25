mod bone;
use std::num::NonZeroUsize;

pub use bone::Bone;
use graphics::linear_algebra::Matrix;

use super::geometry::Pose;
use crate::modelling::cubic::geometry::SkeletalAnimation;
use crate::transform::Transform;

#[derive(Debug, Clone)]
pub struct Skeleton {
    // All accesses to `values` is offset by -1, where 0 means accessing the root node (given in
    // functions), 1 means accessing values[0], 2 => values[1] etc...
    pub(crate) bones: Vec<Bone>,
}

impl Default for Skeleton {
    fn default() -> Self {
        Self::new()
    }
}

impl Skeleton {
    pub const ROOT_BONE: usize = 0;

    pub fn new() -> Self {
        Self { bones: vec![] }
    }

    /// Returns the index of that bone, or `None` if the parent does not exist.
    pub fn push_bone(&mut self, bone: Bone) -> Option<NonZeroUsize> {
        let (GetBone::RootBone | GetBone::Bone { .. }) = self.get(bone.parent) else {
            return None;
        };

        self.bones.push(bone);

        // len is one more than the highest index, but this is correct because the root
        // is at 0
        Some(NonZeroUsize::new(self.bones.len()).expect("just pushed at least one value"))
    }

    pub(crate) fn get(&self, index: usize) -> GetBone<'_> {
        if let Some(non_zero_index) = NonZeroUsize::new(index) {
            match self.bones.get(index - 1) {
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

    /// Used to determine whether a skeleton and an animation are compatible
    pub(crate) fn signature(&self) -> Vec<usize> {
        self.bones.iter().map(|bone| bone.parent).collect()
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

impl<'a> GetBone<'a> {
    pub fn is_bone(&self) -> bool {
        match self {
            Self::NoBone => false,
            Self::RootBone | Self::Bone { .. } => true,
        }
    }
}
