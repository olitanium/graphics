use core::hash::{Hash as _, Hasher as _};
use core::num::NonZeroUsize;
use std::hash::DefaultHasher;

use utils::new;

use crate::modelling::cubic::animation::BoneAnimation;
use crate::modelling::cubic::skeleton::SkeletonBoneIndex;

#[derive(Debug, Default)]
#[must_use = "`SkeletalAnimation` must be built with the `build` method"]
pub struct SkeletalAnimationBuilder {
    /// Used to identify whether this animation is valid for a given skeleton.
    /// The skeleton's signature can be derived as the ordered list of bone
    /// parents.
    signature: Vec<SkeletonBoneIndex>,
    // The 0th bone is the Root bone, and isn't animated. bones[0] refers to skeleton bone 1.
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
        parent_bone: &SkeletonBoneIndex,
    ) -> Result<(), BoneAnimation> {
        if parent_bone.0 <= self.bones.len() {
            // The parent bone is part of the animation.
            self.signature.push(parent_bone.clone_inner());
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
        SkeletalAnimation {
            signature: hash,
            bones: self.bones,
        }
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
