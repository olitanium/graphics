use std::num::NonZeroUsize;

use crate::modelling::cubic::geometry::{BoneAnimation, Pose, animation::bone_animation};

#[derive(Debug, Default)]
pub struct SkeletalAnimation {
    /// Used to identify whether this animation is valid for a given skeleton. The skeleton's signature can be derived as the ordered list of bone parents.
    signature: Vec<usize>,
    // The 0th bone is the Root bone, and isn't animated. bones[0] refers to skeletin bone 1.
    // All accesses to bones must go through get() which returns an option in case of accessing bone 0.
    bones: Vec<BoneAnimation>,
}

impl SkeletalAnimation {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_animation(&mut self, animation: BoneAnimation, parent_bone: usize) -> Result<(), BoneAnimation> {
        if parent_bone <= self.bones.len() {
            // The parent bone is part of the animation.
            self.signature.push(parent_bone);
            self.bones.push(animation);

            Ok(())
        } else {
            Err(animation)
        }
    }

    pub fn get_animation(&self, bone: NonZeroUsize) -> Option<&BoneAnimation> {
        self.bones.get(bone.get() - 1)
    }

    pub fn get_pose(&self, bone: NonZeroUsize, time: f32) -> Option<Pose> {
        eprintln!("trying to get animation bone {bone:?}");
        let animation = self.get_animation(bone);
        eprintln!("got animation: {animation:?} from bone {bone:?}");
     
        animation.map(|anim| anim.get_pose(time))



    }
}