use std::{hash::{DefaultHasher, Hash}, num::NonZeroUsize};
use std::hash::Hasher;
use utils::new;

use crate::modelling::cubic::geometry::BoneAnimation;

#[derive(Debug, Default)]
pub struct SkeletalAnimationBuilder {
    /// Used to identify whether this animation is valid for a given skeleton.
    /// The skeleton's signature can be derived as the ordered list of bone
    /// parents.
    signature: Vec<usize>,
    // The 0th bone is the Root bone, and isn't animated. bones[0] refers to skeletin bone 1.
    // All accesses to bones must go through get() which returns an option in case of accessing
    // bone 0.
    bones: Vec<BoneAnimation>,
}

#[derive(Debug, Clone)]
pub struct SkeletalAnimation {
    signature: u64,
    bones: Vec<BoneAnimation>,
}

impl SkeletalAnimationBuilder {
    new!();

    pub fn push_animation(
        &mut self,
        animation: BoneAnimation,
        parent_bone: usize,
    ) -> Result<(), BoneAnimation> {
        if parent_bone <= self.bones.len() {
            // The parent bone is part of the animation.
            self.signature.push(parent_bone);
            self.bones.push(animation);

            Ok(())
        } else {
            Err(animation)
        }
    }

    pub fn build(self) -> SkeletalAnimation {
        let mut hasher = DefaultHasher::new();
        self.signature.hash(&mut hasher);
        let hash = hasher.finish();
        SkeletalAnimation { signature: hash, bones: self.bones }
    }
}

impl SkeletalAnimation {
    pub fn builder() -> SkeletalAnimationBuilder {
        SkeletalAnimationBuilder::new()
    }
    pub fn get_animation(&self, bone: NonZeroUsize) -> Option<&BoneAnimation> {
        self.bones.get(bone.get() - 1)
    }

    pub const fn signature(&self) -> u64 {
        self.signature
    }
}
