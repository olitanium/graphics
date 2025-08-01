mod bone;
pub use bone::Bone;
use graphics::linear_algebra::Matrix;

use super::geometry::{Pose, YieldsPose};

#[derive(Debug, Clone)]
pub struct Skeleton {
    values: Vec<Bone>,
}

impl Default for Skeleton {
    fn default() -> Self {
        Self::new()
    }
}

impl Skeleton {
    pub fn new() -> Self {
        Self {
            values: vec![Bone::root()],
        }
    }

    /// Returns the index of that bone, or `None` if the parent does not exist.
    pub fn push_bone(&mut self, bone: Bone) -> Option<usize> {
        let parent = bone.parent?;

        self.values.get(parent)?;

        self.values.push(bone);

        Some(self.values.len() - 1)
    }

    pub fn get_bone_mut(&mut self, index: usize) -> Option<&mut Bone> {
        self.values.get_mut(index)
    }

    pub fn set(&mut self, index: usize, pose: Pose) -> Result<(), Pose> {
        let Some(bone) = self.get_bone_mut(index) else {
            return Err(pose);
        };
        bone.set(pose);
        Ok(())
    }

    pub fn get_all_bones(&self, animation: usize, time: f32) -> Vec<Matrix<4, 4>> {
        self.values
            .iter()
            .map(|bone| bone.get_pose((&self.values, animation, time)).as_matrix())
            .collect()
    }
}

impl YieldsPose for Skeleton {
    type Hint = (
        bool,  // realtive
        usize, // bone
        usize, // animation
        f32,   // time
    );

    fn get_pose(&self, hint: Self::Hint) -> Pose {
        let (relative, bone, animation, time) = hint;

        self.values
            .get(bone)
            .map(|bone| {
                let mut bone_pose = bone.get_pose((&self.values, animation, time));
                if relative {
                    bone_pose = bone_pose.apply_after(bone.inverse_bind_pose);
                }
                bone_pose
            })
            .unwrap_or_default()
    }
}
