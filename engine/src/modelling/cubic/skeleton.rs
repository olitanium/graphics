mod bone;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::num::NonZeroUsize;

pub use bone::Bone;
use bone::BoneBuilder;
use utils::new;

#[derive(Debug, Clone, Default)]
pub struct SkeletonBuilder {
    // All accesses to `values` is offset by -1, where 0 means accessing the root node (given in
    // functions), 1 means accessing values[0], 2 => values[1] etc...
    pub(crate) bones: Vec<Bone>,
}

impl SkeletonBuilder {
    new!();

    /// Returns the index of that bone, or `None` if the parent does not exist.
    pub fn push_bone(&mut self, bone: BoneBuilder) -> Option<SkeletonBoneIndex> {
        let bone = bone.build(self)?;

        self.bones.push(bone);

        // len is one more than the highest index, but this is correct because the root
        // is at 0
        Some(SkeletonBoneIndex(self.bones.len()))
    }

    pub(crate) fn get(&self, index: SkeletonBoneIndex) -> GetBone<'_> {
        if index.0 != 0 {
            match self.bones.get(index.0 - 1) {
                Some(bone) => GetBone::Bone { bone },
                None => GetBone::NoBone,
            }
        } else {
            // i.e. index == 0
            GetBone::RootBone
        }
    }

    pub fn build(self) -> Skeleton {
        let mut hasher = DefaultHasher::new();
        let signature: Vec<_> = self.bones.iter().map(|bone| bone.parent).collect();
        signature.hash(&mut hasher);
        let hash = hasher.finish();
        Skeleton {
            bones: self.bones,
            signature: hash,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Skeleton {
    bones: Vec<Bone>,
    signature: u64,
}

impl Default for Skeleton {
    fn default() -> Self {
        Self::builder().build()
    }
}

impl Skeleton {
    pub fn builder() -> SkeletonBuilder {
        SkeletonBuilder::new()
    }

    pub(crate) fn get(&self, index: SkeletonBoneIndex) -> GetBone<'_> {
        if index.0 != 0 {
            match self.bones.get(index.0 - 1) {
                Some(bone) => GetBone::Bone { bone },
                None => GetBone::NoBone,
            }
        } else {
            // i.e. index == 0
            GetBone::RootBone
        }
    }

    /// Used to determine whether a skeleton and an animation are compatible
    pub(crate) const fn signature(&self) -> u64 {
        self.signature
        // self.bones.iter().map(|bone| bone.parent).collect()
    }

    pub(crate) const fn len_incl_root(&self) -> usize {
        self.bones.len() + 1
    }

    pub(crate) fn iter_non_root(&self) -> impl Iterator<Item = (NonZeroUsize, &Bone)> {
        self.bones.iter().enumerate().map(|(index, bone)| {
            (
                NonZeroUsize::new(index + 1).expect("x+1 > 0 for all x, no overflow"),
                bone,
            )
        })
    }
}

pub enum GetBone<'a> {
    NoBone,
    RootBone,
    Bone { bone: &'a Bone },
}

impl<'a> GetBone<'a> {
    pub const fn is_bone(&self) -> bool {
        match self {
            Self::NoBone => false,
            Self::RootBone | Self::Bone { .. } => true,
        }
    }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct SkeletonBoneIndex(pub(crate) usize);

impl SkeletonBoneIndex {
    pub const SKELETON_ROOT_BONE: Self = Self(0);
}
