mod bone;
pub use bone::Bone;
use graphics::linear_algebra::Matrix;

use super::geometry::Pose;
use crate::transform::Transform;

#[derive(Debug, Clone)]
pub struct Skeleton {
    pub root: Pose,
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
            root: Pose::identity(),
            values: vec![],
        }
    }

    /// Returns the index of that bone, or `None` if the parent does not exist.
    pub fn push_bone(&mut self, bone: Bone) -> Option<usize> {
        let (GetBone::RootBone | GetBone::Bone(_)) = self.get(bone.parent) else {
            return None;
        };

        self.values.push(bone);

        // len is one more than the highest index, but this offset is taken care of in
        // Skeleton::get
        Some(self.values.len())
    }

    // pub fn get_root_mut(&mut self) -> &mut Pose {
    // &mut self.root
    // }

    pub fn get_all_bones(&self, animation: usize, time: f32) -> Vec<Matrix<4, 4>> {
        let mut output = vec![self.root.as_matrix()];

        let extend = self
            .values
            .iter()
            .map(|bone| bone.get_pose(self, animation, time).as_matrix());

        output.extend(extend);
        output
    }

    pub(crate) fn get(&self, index: usize) -> GetBone<'_> {
        if index == 0 {
            GetBone::RootBone
        } else {
            match self.values.get(index - 1) {
                Some(bone) => GetBone::Bone(bone),
                None => GetBone::NoBone,
            }
        }
    }

    pub fn get_bone_pose(&self, relative: bool, bone: usize, animation: usize, time: f32) -> Pose {
        match self.get(bone) {
            GetBone::Bone(bone) => {
                let pose = bone.get_pose(self, animation, time);

                if relative {
                    pose.transform(&bone.inverse_bind_pose)
                } else {
                    pose
                }
            }
            GetBone::RootBone => self.root,
            GetBone::NoBone => Pose::default(),
        }

        //         .map_or_default(|maybe_bone| {
        // maybe_bone.map_or(self.root, |bone| {
        // let pose = bone.get_pose(self, animation, time);
        //
        // if relative {
        // pose.transform(&bone.inverse_bind_pose)
        // } else {
        // pose
        // }
        // })
        // })
    }
}

pub enum GetBone<'a> {
    NoBone,
    RootBone,
    Bone(&'a Bone),
}
