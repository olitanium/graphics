mod bone;
use core::hash::{Hash, Hasher as _};
use core::num::NonZeroUsize;
use std::hash::DefaultHasher;
use std::sync::LazyLock;

pub use bone::Bone;
use bone::BoneBuilder;
use utils::new;

#[derive(Debug, Clone, Default)]
#[must_use = "Model must be created with the `build` method"]
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

        // len is one more than the highest index, but this is correct because
        // the root is at 0
        Some(SkeletonBoneIndex(self.bones.len()))
    }

    pub(crate) fn get<'a>(&'a self, index: &SkeletonBoneIndex) -> GetBone<'a> {
        if index.0 == 0 {
            GetBone::RootBone
        } else {
            self.bones
                .get(index.0 - 1)
                .map_or(GetBone::NoBone, |bone| GetBone::Bone { bone })
        }
    }

    pub fn build(self) -> Skeleton {
        let mut hasher = DefaultHasher::new();
        #[expect(clippy::collection_is_never_read, reason = "clippy bug")]
        let signature: Vec<_> = self
            .bones
            .iter()
            .map(|bone| bone.parent.clone_inner())
            .collect();
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

    pub fn just_root() -> &'static Self {
        static LAZY: LazyLock<Skeleton> = LazyLock::new(|| SkeletonBuilder::new().build());

        &LAZY
    }

    pub(crate) fn get<'a>(&'a self, index: &SkeletonBoneIndex) -> GetBone<'a> {
        if index.0 == 0 {
            GetBone::RootBone
        } else {
            self.bones
                .get(index.0 - 1)
                .map_or(GetBone::NoBone, |bone| GetBone::Bone { bone })
        }
    }

    /// Used to determine whether a skeleton and an animation are compatible.
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

impl GetBone<'_> {
    pub const fn is_bone(&self) -> bool {
        match self {
            Self::NoBone => false,
            Self::RootBone | Self::Bone { .. } => true,
        }
    }
}

#[derive(Debug, Hash, Eq, PartialEq)]
pub struct SkeletonBoneIndex(pub(crate) usize);

impl SkeletonBoneIndex {
    pub const SKELETON_ROOT_BONE: Self = Self(0);

    pub(crate) const fn clone_inner(&self) -> Self {
        Self(self.0)
    }
}
