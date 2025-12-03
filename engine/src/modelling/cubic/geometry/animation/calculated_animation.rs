use std::num::NonZeroUsize;

use crate::TransformBy;
use crate::entity::{AggregateModelIndex, EntityInner};
use crate::modelling::cubic::geometry::{AnimationError, Pose, SkeletalAnimation};
use crate::modelling::cubic::skeleton::SkeletonBoneIndex;
use crate::modelling::{CubicVertex, Skeleton};
use crate::transform::Transform;

#[derive(Debug, Clone)]
pub struct CalculatedAnimation {
    poses: Vec<Pose>,
    signature: Option<u64>,
}

pub struct CalculatedAnimationBuilder<'a> {
    // Invariant: all animation units can only point to those before themselves.
    // this is enforced by the `push` method
    units: Vec<AnimationUnit<'a>>,
}

impl CalculatedAnimation {
    pub fn builder<'a>(
        root_pose: Pose,
        root_skeleton: &'a Skeleton,
        root_animation: Option<(&'a SkeletalAnimation, f32)>,
    ) -> Result<CalculatedAnimationBuilder<'a>, AnimationError> {
        CalculatedAnimationBuilder::new(root_pose, root_skeleton, root_animation)
    }

    pub fn poses(&self, skeleton: &Skeleton) -> Result<&[Pose], AnimationError> {
        if let Some(signature) = self.signature {
            if skeleton.signature() == signature {
                Ok(&self.poses)
            } else {
                Err(AnimationError::SignaturesDoNotMatch {
                    animation: signature,
                    skeleton: skeleton.signature(),
                })
            }
        } else {
            Ok(&self.poses)
        }
    }

    pub fn get(&self, index: SkeletonBoneIndex) -> Result<Pose, AnimationError> {
        self.poses
            .get(index.0)
            .copied()
            .ok_or(AnimationError::BoneDoesNotExist {
                requested: index,
                length: self.poses.len(),
            })
    }
}

struct Parent {
    index: AggregateModelIndex,
    bone_attachment_point: SkeletonBoneIndex,
}

/// All Animation units can only refer to units before them in the heirarchy.
/// This is enforced by the `push` method
struct AnimationUnit<'a> {
    parent: Option<Parent>,
    local_transform: Pose,
    // invariant: Skeleton and Skeletal Animation have same signature
    skeleton: &'a Skeleton,
    animation: Option<(&'a SkeletalAnimation, f32)>,
}

#[derive(Debug, Copy, Clone)]
struct PoseAndInverseBindPose {
    pose: Pose,
    inverse_bind_pose: Pose,
}

impl<'a> CalculatedAnimationBuilder<'a> {
    pub fn new(
        root_pose: Pose,
        root_skeleton: &'a Skeleton,
        root_animation: Option<(&'a SkeletalAnimation, f32)>,
    ) -> Result<Self, AnimationError> {
        match root_animation {
            Some((animation, _)) if animation.signature() != root_skeleton.signature() => {
                Err(AnimationError::SignaturesDoNotMatch {
                    animation: animation.signature(),
                    skeleton: root_skeleton.signature(),
                })
            }
            other => Ok(Self {
                units: vec![AnimationUnit {
                    parent: None,
                    local_transform: root_pose,
                    skeleton: root_skeleton,
                    animation: other,
                }],
            }),
        }
    }

    pub fn push(
        &mut self,
        parent: AggregateModelIndex,
        attachment_point: SkeletonBoneIndex,
        local_transform: Pose,
        skeleton: &'a Skeleton,
        animation: Option<(&'a SkeletalAnimation, f32)>,
    ) -> Result<NonZeroUsize, AnimationError> {
        let output_index = self.units.len();

        if let Some(parent_skeleton) = self.units.get(parent.0) {
            if parent_skeleton.skeleton.get(attachment_point).is_bone() {
                // Asserted that there is a valid parent and that parent has a bone to attach
                // to.
                // parent IS in the builder and skeleton DOES have relevant attachement point
                // (bone)
                match animation {
                    Some((in_animation, _)) if in_animation.signature() != skeleton.signature() => {
                        Err(AnimationError::SignaturesDoNotMatch {
                            animation: in_animation.signature(),
                            skeleton: skeleton.signature(),
                        })
                    }
                    other => {
                        self.units.push(AnimationUnit {
                            parent: Some(Parent {
                                index: parent,
                                bone_attachment_point: attachment_point,
                            }),
                            local_transform,
                            skeleton,
                            animation: other,
                        });

                        Ok(NonZeroUsize::new(output_index)
                            .expect("should be at least one element (from creation) so `len` > 0"))
                    }
                }
            } else {
                Err(AnimationError::BoneInParentDoesNotExist {
                    unit: parent,
                    length: parent_skeleton.skeleton.len_incl_root(),
                    requested: attachment_point,
                })
            }
        } else {
            Err(AnimationError::ParentDoesNotExist {
                requested: parent,
                available: output_index - 1,
            })
        }
    }

    fn get_world_positions(self) -> Vec<(Vec<PoseAndInverseBindPose>, Option<u64>)> {
        let mut output: Vec<(Vec<PoseAndInverseBindPose>, Option<u64>)> = Vec::new();

        for unit in self.units {
            let root = if let Some(parent) = unit.parent {
                let parent_setup = output.get(parent.index.0).expect(
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

            let poses = if let Some((animation, time)) = unit.animation {
                let mut poses = vec![PoseAndInverseBindPose {
                    pose: root,
                    inverse_bind_pose: Pose::IDENTITY,
                }];

                for (index, bone) in unit.skeleton.iter_non_root() {
                    // NOTE: all bone parents are above it in the heirarchy, so we can look up the
                    // total parent pose in the pre-filled `poses` vec.

                    let PoseAndInverseBindPose { pose: parent, .. } =
                        *poses.get(bone.parent.0).expect(
                            "Skeleton invariant, each bone in a skeleton may only refer to bones \
                             before it",
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

                (poses, Some(animation.signature()))
            } else {
                (
                    vec![
                        PoseAndInverseBindPose {
                            pose: root,
                            inverse_bind_pose: Pose::IDENTITY,
                        };
                        unit.skeleton.len_incl_root()
                    ],
                    None,
                )
            };

            output.push(poses);
        }

        output
    }

    pub fn build_fixing_bone(
        self,
        unit: AggregateModelIndex,
        bone: SkeletonBoneIndex,
        pose: Pose,
    ) -> Result<Vec<CalculatedAnimation>, AnimationError> {
        let world_positions = self.get_world_positions();

        let poses_this_unit = &world_positions
            .get(unit.0)
            .ok_or_else(|| AnimationError::UnitDoesNotExist {
                available: world_positions.len(),
                requested: unit,
            })?
            .0;
        let fixed_pose = poses_this_unit
            .get(bone.0)
            .ok_or_else(|| AnimationError::BoneInParentDoesNotExist {
                unit,
                requested: bone,
                length: poses_this_unit.len(),
            })?
            .pose;

        let deviation = pose.transform(&fixed_pose.inverse());

        Ok(world_positions
            .into_iter()
            .map(|(vec, opt_signature)| {
                let poses = vec
                    .into_iter()
                    .map(
                        |PoseAndInverseBindPose {
                             pose,
                             inverse_bind_pose,
                         }| {
                            deviation.transform(&pose).transform(&inverse_bind_pose)
                        },
                    )
                    .collect();

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
                let poses = vec
                    .into_iter()
                    .map(
                        |PoseAndInverseBindPose {
                             pose,
                             inverse_bind_pose,
                         }| pose.transform(&inverse_bind_pose),
                    )
                    .collect();

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
) -> Result<Vec<(&'a M, CalculatedAnimation)>, AnimationError>
where
    M::Vertex: CubicVertex,
{
    input
        .into_iter()
        .map(|(cubic, root, animation)| {
            Ok((
                cubic,
                CalculatedAnimation::builder(root, cubic.skeleton(), animation)?
                    .build()
                    .swap_remove(0), // select 0 because no aggregare entities exist yet
            ))
        })
        .collect()
}
