use geometry::{Pose, Transform};
use utils::error_boilerplate;

use crate::entity::AggregateModelIndex;
use crate::modelling::Skeleton;
use crate::modelling::cubic::SkeletonBoneIndex;
use crate::modelling::cubic::animation::{
    AnimationUnit,
    CalcAnimUnitIndex,
    CalculatedAnimation,
    Parent,
    PoseAndInverseBindPose,
    SignaturesDoNotMatchError,
    SkeletalAnimation,
};

#[derive(Debug, Clone)]
pub struct AggregateCalcAnim {
    pub(crate) vec: Vec<CalculatedAnimation>,
}

#[derive(Debug)]
pub struct AggregateCalcAnimBuilder<'a> {
    // Invariant: all animation units can only point to those before themselves.
    // this is enforced by the `push` method
    units: Vec<AnimationUnit<'a>>,
}

impl AggregateCalcAnim {
    pub fn builder<'a>(
        root_pose: Pose,
        root_skeleton: &'a Skeleton,
        root_animation: Option<(&'a SkeletalAnimation, f32)>,
    ) -> Result<AggregateCalcAnimBuilder<'a>, SignaturesDoNotMatchError> {
        AggregateCalcAnimBuilder::new(root_pose, root_skeleton, root_animation)
    }

    pub fn get<'a>(&'a self, index: &AggregateModelIndex) -> Option<&'a CalculatedAnimation> {
        self.vec.get(index.0)
    }

    pub fn root(&self) -> &CalculatedAnimation {
        self.vec.first().expect("always as root pose")
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum PushAnimationError {
    SignaturesDoNotMatch(SignaturesDoNotMatchError),
    ParentUnitDoesNotExist,
    BoneInParentDoesNotExist,
}
error_boilerplate!(PushAnimationError);

impl<'a> AggregateCalcAnimBuilder<'a> {
    pub fn new(
        root_pose: Pose,
        root_skeleton: &'a Skeleton,
        root_animation: Option<(&'a SkeletalAnimation, f32)>,
    ) -> Result<Self, SignaturesDoNotMatchError> {
        let unit = AnimationUnit::new(None, root_pose, root_skeleton, root_animation)?;
        Ok(Self { units: vec![unit] })
    }
}

impl<'a> AggregateCalcAnimBuilder<'a> {
    pub(crate) fn push(
        &mut self,
        parent_index: CalcAnimUnitIndex,
        attachment_point: &SkeletonBoneIndex,
        local_transform: Pose,
        skeleton: &'a Skeleton,
        animation: Option<(&'a SkeletalAnimation, f32)>,
    ) -> Result<CalcAnimUnitIndex, PushAnimationError> {
        let output_index = self.units.len();

        if let Some(parent_skeleton) = self.units.get(parent_index.index) {
            if parent_skeleton.skeleton.get(attachment_point).is_bone() {
                // Asserted that there is a valid parent and that parent has a
                // bone to attach to.
                // parent IS in the builder and skeleton DOES have relevant
                // attachement point (bone)
                let parent = Some(Parent {
                    index: parent_index,
                    bone_attachment_point: attachment_point.clone_inner(),
                });
                let unit = AnimationUnit::new(parent, local_transform, skeleton, animation)
                    .map_err(PushAnimationError::SignaturesDoNotMatch)?;

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

impl<'a> AggregateCalcAnimBuilder<'a> {
    fn get_world_positions(self) -> Vec<(Vec<PoseAndInverseBindPose>, u64)> {
        let mut output: Vec<(Vec<PoseAndInverseBindPose>, u64)> = Vec::new();

        for unit in self.units {
            let root = if let Some(parent) = &unit.parent {
                let parent_setup = output.get(parent.index.index).expect(
                    "invariant, parent should exist earlier in the list than the descendent",
                );
                let attachment_pose = parent_setup
                    .0
                    .get(parent.bone_attachment_point.0)
                    .expect("invariant, the existence of this is checked by the `push` method");
                attachment_pose.pose.transform(unit.local_transform)
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
#[non_exhaustive]
pub enum BuildFixingBoneError {
    UnitDoesNotExist,
    BoneDoesNotExist,
}

error_boilerplate!(BuildFixingBoneError);

impl<'a> AggregateCalcAnimBuilder<'a> {
    pub(crate) fn build_fixing_bone(
        self,
        unit: CalcAnimUnitIndex,
        bone: &SkeletonBoneIndex,
        pose: Pose,
    ) -> Result<AggregateCalcAnim, BuildFixingBoneError> {
        let world_positions = self.get_world_positions();

        let poses_this_unit = &world_positions
            .get(unit.index)
            .ok_or(BuildFixingBoneError::UnitDoesNotExist)?
            .0;

        let fixed_pose = poses_this_unit
            .get(bone.0)
            .ok_or(BuildFixingBoneError::BoneDoesNotExist)?
            .pose;

        let deviation = pose.transform(fixed_pose.inverse());

        let vec = world_positions
            .into_iter()
            .map(|(vec, opt_signature)| {
                let poses = PoseAndInverseBindPose::process_vec_with_deviation(vec, deviation);

                CalculatedAnimation {
                    poses,
                    signature: opt_signature,
                }
            })
            .collect();

        Ok(AggregateCalcAnim { vec })
    }

    pub fn build(self) -> AggregateCalcAnim {
        let world_positions = self.get_world_positions();
        let vec = world_positions
            .into_iter()
            .map(|(vec, opt_signature)| {
                let poses = PoseAndInverseBindPose::process_vec(vec);

                CalculatedAnimation {
                    poses,
                    signature: opt_signature,
                }
            })
            .collect();
        AggregateCalcAnim { vec }
    }
}
