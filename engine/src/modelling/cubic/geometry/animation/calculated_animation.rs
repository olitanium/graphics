use utils::error_boilerplate;

use crate::TransformBy;
use crate::entity::EntityInner;
use crate::modelling::cubic::geometry::{Pose, SkeletalAnimation};
use crate::modelling::cubic::skeleton::SkeletonBoneIndex;
use crate::modelling::{CubicVertex, Skeleton};
use crate::transform::Transform;

#[derive(Debug, Clone)]
pub struct CalculatedAnimation {
    poses: Vec<Pose>,
    signature: u64,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct CalcAnimUnitIndex {
    pub(crate) index: usize,
}

impl CalcAnimUnitIndex {
    pub(crate) const ROOT_UNIT_INDEX: Self = Self { index: 0 };
}

#[derive(Debug)]
pub struct CalculatedAnimationBuilder<'a> {
    // Invariant: all animation units can only point to those before themselves.
    // this is enforced by the `push` method
    units: Vec<AnimationUnit<'a>>,
}

impl CalculatedAnimation {
    pub fn once<'a>(
        root_pose: Pose,
        root_skeleton: &'a Skeleton,
        root_animation: Option<(&'a SkeletalAnimation, f32)>,
    ) -> Result<Self, SignaturesDoNotMatchError> {
        CalculatedAnimationBuilder::once(root_pose, root_skeleton, root_animation)
    }

    pub fn builder<'a>(
        root_pose: Pose,
        root_skeleton: &'a Skeleton,
        root_animation: Option<(&'a SkeletalAnimation, f32)>,
    ) -> Result<CalculatedAnimationBuilder<'a>, SignaturesDoNotMatchError> {
        CalculatedAnimationBuilder::new(root_pose, root_skeleton, root_animation)
    }

    pub fn poses(&self, skeleton: &Skeleton) -> Result<&[Pose], SignaturesDoNotMatchError> {
        if skeleton.signature() == self.signature {
            Ok(&self.poses)
        } else {
            Err(SignaturesDoNotMatchError)
        }
    }
}

#[derive(Debug)]
pub enum GetAnimationError {
    SignaturesDoNotMatch(SignaturesDoNotMatchError),
    BoneDoesNotExist,
}

error_boilerplate!(GetAnimationError);

impl CalculatedAnimation {
    pub fn get(
        &self,
        index: SkeletonBoneIndex,
        skeleton: &Skeleton,
    ) -> Result<Pose, GetAnimationError> {
        self.poses(skeleton)
            .map_err(|err| GetAnimationError::SignaturesDoNotMatch(err))?
            .get(index.0)
            .copied()
            .ok_or(GetAnimationError::BoneDoesNotExist)
    }
}

#[derive(Debug, Clone, Copy)]
struct Parent {
    index: CalcAnimUnitIndex,
    bone_attachment_point: SkeletonBoneIndex,
}

/// All Animation units can only refer to units before them in the heirarchy.
/// This is enforced by the `push` method
#[derive(Debug)]
struct AnimationUnit<'a> {
    parent: Option<Parent>,
    local_transform: Pose,
    // invariant: Skeleton and Skeletal Animation have same signature
    skeleton: &'a Skeleton,
    animation: Option<(&'a SkeletalAnimation, f32)>,
}

#[derive(Debug)]
pub struct SignaturesDoNotMatchError;
error_boilerplate!(SignaturesDoNotMatchError);

impl<'a> AnimationUnit<'a> {
    fn new(
        parent: Option<Parent>,
        root_pose: Pose,
        root_skeleton: &'a Skeleton,
        root_animation: Option<(&'a SkeletalAnimation, f32)>,
    ) -> Result<AnimationUnit<'a>, SignaturesDoNotMatchError> {
        match root_animation {
            Some((animation, _)) if animation.signature() != root_skeleton.signature() => {
                Err(SignaturesDoNotMatchError)
            }
            other => Ok(AnimationUnit {
                parent,
                local_transform: root_pose,
                skeleton: root_skeleton,
                animation: other,
            }),
        }
    }

    fn calculate_animation(self, root: Pose) -> (Vec<PoseAndInverseBindPose>, u64) {
        let poses = if let Some((animation, time)) = self.animation {
            let mut poses = vec![PoseAndInverseBindPose {
                pose: root,
                inverse_bind_pose: Pose::IDENTITY,
            }];

            for (index, bone) in self.skeleton.iter_non_root() {
                // NOTE: all bone parents are above it in the heirarchy, so we can look up the
                // total parent pose in the pre-filled `poses` vec.

                let PoseAndInverseBindPose { pose: parent, .. } = *poses.get(bone.parent.0).expect(
                    "Skeleton invariant, each bone in a skeleton may only refer to bones before it",
                );
                let total = animation
                    .get_animation(index)
                    .expect("invariant, signatures are the same so should exist")
                    .get_pose(time)
                    .transform_by(&parent);
                let inverse_bind_pose = bone.inverse_bind_pose();

                poses.push(PoseAndInverseBindPose {
                    pose: total,
                    inverse_bind_pose,
                });
            }

            (poses, self.skeleton.signature())
        } else {
            (
                vec![
                    PoseAndInverseBindPose {
                        pose: root,
                        inverse_bind_pose: Pose::IDENTITY,
                    };
                    self.skeleton.len_incl_root()
                ],
                self.skeleton.signature(),
            )
        };

        poses
    }
}

#[derive(Debug, Copy, Clone)]
struct PoseAndInverseBindPose {
    pose: Pose,
    inverse_bind_pose: Pose,
}

impl PoseAndInverseBindPose {
    fn process_vec(this: Vec<Self>) -> Vec<Pose> {
        this.into_iter()
            .map(
                |PoseAndInverseBindPose {
                     pose,
                     inverse_bind_pose,
                 }| pose.transform(&inverse_bind_pose),
            )
            .collect()
    }

    fn process_vec_with_deviation(this: Vec<Self>, deviation: Pose) -> Vec<Pose> {
        this.into_iter()
            .map(
                |PoseAndInverseBindPose {
                     pose,
                     inverse_bind_pose,
                 }| deviation.transform(&pose).transform(&inverse_bind_pose),
            )
            .collect()
    }
}

impl<'a> CalculatedAnimationBuilder<'a> {
    pub fn once(
        root_pose: Pose,
        root_skeleton: &'a Skeleton,
        root_animation: Option<(&'a SkeletalAnimation, f32)>,
    ) -> Result<CalculatedAnimation, SignaturesDoNotMatchError> {
        let unit = AnimationUnit::new(None, root_pose, root_skeleton, root_animation)?;
        let root = unit.local_transform;

        let (vec_pose, signature) = unit.calculate_animation(root);

        let poses = PoseAndInverseBindPose::process_vec(vec_pose);

        Ok(CalculatedAnimation { poses, signature })
    }

    pub fn new(
        root_pose: Pose,
        root_skeleton: &'a Skeleton,
        root_animation: Option<(&'a SkeletalAnimation, f32)>,
    ) -> Result<Self, SignaturesDoNotMatchError> {
        let unit = AnimationUnit::new(None, root_pose, root_skeleton, root_animation)?;
        Ok(Self { units: vec![unit] })
    }
}

#[derive(Debug)]
pub enum PushAnimationError {
    SignaturesDoNotMatch(SignaturesDoNotMatchError),
    ParentUnitDoesNotExist,
    BoneInParentDoesNotExist,
}
error_boilerplate!(PushAnimationError);

impl<'a> CalculatedAnimationBuilder<'a> {
    pub(crate) fn push(
        &mut self,
        parent_index: CalcAnimUnitIndex,
        attachment_point: SkeletonBoneIndex,
        local_transform: Pose,
        skeleton: &'a Skeleton,
        animation: Option<(&'a SkeletalAnimation, f32)>,
    ) -> Result<CalcAnimUnitIndex, PushAnimationError> {
        let output_index = self.units.len();

        if let Some(parent_skeleton) = self.units.get(parent_index.index) {
            if parent_skeleton.skeleton.get(attachment_point).is_bone() {
                // Asserted that there is a valid parent and that parent has a bone to attach
                // to.
                // parent IS in the builder and skeleton DOES have relevant attachement point
                // (bone)
                let parent = Some(Parent {
                    index: parent_index,
                    bone_attachment_point: attachment_point,
                });
                let unit = AnimationUnit::new(parent, local_transform, skeleton, animation)
                    .map_err(|error| PushAnimationError::SignaturesDoNotMatch(error))?;
                self.units.push(unit);
                Ok(CalcAnimUnitIndex {
                    index: output_index,
                })
            } else {
                Err(PushAnimationError::BoneInParentDoesNotExist)
            }
        } else {
            Err(PushAnimationError::ParentUnitDoesNotExist)
        }
    }
}

impl<'a> CalculatedAnimationBuilder<'a> {
    fn get_world_positions(self) -> Vec<(Vec<PoseAndInverseBindPose>, u64)> {
        let mut output: Vec<(Vec<PoseAndInverseBindPose>, u64)> = Vec::new();

        for unit in self.units {
            let root = if let Some(parent) = unit.parent {
                let parent_setup = output.get(parent.index.index).expect(
                    "invariant, parent should exist earlier in the list than the descendent",
                );
                let attachment_pose = parent_setup
                    .0
                    .get(parent.bone_attachment_point.0)
                    .expect("invariant, the existence of this is checked by the `push` method");
                attachment_pose.pose.transform(&unit.local_transform)
            } else {
                unit.local_transform
            };

            let poses = unit.calculate_animation(root);

            output.push(poses);
        }

        output
    }
}

#[derive(Debug)]
pub enum BuildFixingBoneError {
    UnitDoesNotExist,
    BoneDoesNotExist,
}

error_boilerplate!(BuildFixingBoneError);

impl<'a> CalculatedAnimationBuilder<'a> {
    pub(crate) fn build_fixing_bone(
        self,
        unit: CalcAnimUnitIndex,
        bone: SkeletonBoneIndex,
        pose: Pose,
    ) -> Result<Vec<CalculatedAnimation>, BuildFixingBoneError> {
        let world_positions = self.get_world_positions();

        let poses_this_unit = &world_positions
            .get(unit.index)
            .ok_or(BuildFixingBoneError::UnitDoesNotExist)?
            .0;

        let fixed_pose = poses_this_unit
            .get(bone.0)
            .ok_or(BuildFixingBoneError::BoneDoesNotExist)?
            .pose;

        let deviation = pose.transform(&fixed_pose.inverse());

        Ok(world_positions
            .into_iter()
            .map(|(vec, opt_signature)| {
                let poses = PoseAndInverseBindPose::process_vec_with_deviation(vec, deviation);

                CalculatedAnimation {
                    poses,
                    signature: opt_signature,
                }
            })
            .collect())
    }

    pub fn build(self) -> Vec<CalculatedAnimation> {
        let world_positions = self.get_world_positions();
        world_positions
            .into_iter()
            .map(|(vec, opt_signature)| {
                let poses = PoseAndInverseBindPose::process_vec(vec);

                CalculatedAnimation {
                    poses,
                    signature: opt_signature,
                }
            })
            .collect()
    }
}

pub(crate) fn process_vec_noaggregate<'a, M: EntityInner>(
    input: Vec<(&'a M, Pose, Option<(&'a SkeletalAnimation, f32)>)>,
) -> Result<Vec<(&'a M, CalculatedAnimation)>, SignaturesDoNotMatchError>
where
    M::Vertex: CubicVertex,
{
    input
        .into_iter()
        .map(|(cubic, root, animation)| {
            Ok((
                cubic,
                CalculatedAnimation::once(root, cubic.skeleton(), animation)?,
            ))
        })
        .collect()
}
