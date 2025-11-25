use std::num::NonZeroUsize;

use crate::EngResult;
use crate::modelling::cubic::geometry::{AnimationError, Pose, SkeletalAnimation};
use crate::modelling::{SimpleCubic, Skeleton};
use crate::transform::Transform;

#[derive(Debug, Clone)]
pub struct CalculatedAnimation {
    offsets: Vec<usize>,
    poses: Vec<Pose>,
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
        root_animation: Option<&'a SkeletalAnimation>,
    ) -> Result<CalculatedAnimationBuilder<'a>, AnimationError> {
        CalculatedAnimationBuilder::new(root_pose, root_skeleton, root_animation)
    }

    pub fn poses(&self) -> &[Pose] {
        &self.poses
    }

    pub fn get(&self, index: usize) -> Result<Pose, AnimationError> {
        self.poses
            .get(index)
            .copied()
            .ok_or(AnimationError::BoneDoesNotExist {
                requested: index,
                length: self.poses.len(),
            })
    }

    pub fn offsets(&self) -> &[usize] {
        &self.offsets
    }
}

struct Parent {
    index: usize,
    attachment_point: usize,
}

/// All Animation units can only refer to units before them in the heirarchy.
/// This is enforced by the `push` method
struct AnimationUnit<'a> {
    parent: Option<Parent>,
    local_transform: Pose,
    // invariant: Skeleton and Skeletal Animation have same signature
    skeleton: &'a Skeleton,
    animation: Option<&'a SkeletalAnimation>,
}

impl<'a> CalculatedAnimationBuilder<'a> {
    pub fn new(
        root_pose: Pose,
        root_skeleton: &'a Skeleton,
        root_animation: Option<&'a SkeletalAnimation>,
    ) -> Result<Self, AnimationError> {
        match root_animation {
            Some(animation) if animation.signature() != root_skeleton.signature() => {
                Err(AnimationError::SignaturesDoNotMatch {
                    animation: animation.signature().to_vec(),
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

    pub fn push<NEVER>(
        &mut self,
        parent: usize,
        attachment_point: usize,
        local_transform: Pose,
        skeleton: &'a Skeleton,
        animation: Option<&'a SkeletalAnimation>,
    ) -> Result<NonZeroUsize, AnimationError> {
        let output_index = self.units.len();

        if let Some(parent_skeleton) = self.units.get(parent) {
            if parent_skeleton.skeleton.get(attachment_point).is_bone() {
                // Asserted that there is a valid parent and that parent has a bone to attach
                // to.
                // parent IS in the builder and skeleton DOES have relevant attachement point
                // (bone)
                match animation {
                    Some(in_animation) if in_animation.signature() != skeleton.signature() => {
                        Err(AnimationError::SignaturesDoNotMatch {
                            animation: in_animation.signature().to_vec(),
                            skeleton: skeleton.signature(),
                        })
                    }
                    other => {
                        self.units.push(AnimationUnit {
                            parent: Some(Parent {
                                index: parent,
                                attachment_point,
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
                Err(AnimationError::AttachementPointDoesNotExist {
                    parent,
                    available: parent_skeleton.skeleton.bones.len(),
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

    pub fn build(self, time: f32) -> CalculatedAnimation {
        #[derive(Debug, Copy, Clone)]
        struct PoseAndInverseBindPose {
            pose: Pose,
            inverse_bind_pose: Pose,
        }

        let mut output: Vec<Vec<PoseAndInverseBindPose>> = Vec::new();

        for unit in self.units {
            let root = if let Some(parent) = unit.parent {
                let parent_setup = output.get(parent.index).expect(
                    "invariant, parent should exist earlier in the list than the descendent",
                );
                let attachment_pose = parent_setup
                    .get(parent.attachment_point)
                    .expect("invariant, the existence of this is checked by the `push` method");
                unit.local_transform.transform(&attachment_pose.pose)
            } else {
                unit.local_transform
            };

            let mut poses = vec![PoseAndInverseBindPose {
                pose: root,
                inverse_bind_pose: Pose::IDENTITY,
            }];

            for (index, bone) in unit.skeleton.bones.iter().enumerate().map(|(index, bone)| {
                (
                    NonZeroUsize::new(index + 1).expect("x + 1 never zero, will never wrap"),
                    bone,
                )
            }) {
                // NOTE: all bone parents are above it in the heirarchy, so we can look up the
                // total parent pose in the pre-filled `poses` vec.

                let PoseAndInverseBindPose { pose: parent, .. } = *poses.get(bone.parent).expect(
                    "Skeleton invariant, each bone in a skeleton may only refer to bones before it",
                );
                let (total, inverse_bind_pose) = if let Some(animation) = unit.animation {
                    let total = animation
                        .get_animation(index)
                        .expect("invariant, signatures are the same so should exist")
                        .get_pose(time)
                        .transform(&parent);
                    let inverse_bind_pose = bone.inverse_bind_pose();
                    (total, inverse_bind_pose)
                } else {
                    (parent, Pose::IDENTITY)
                };

                poses.push(PoseAndInverseBindPose {
                    pose: total,
                    inverse_bind_pose,
                });
            }

            output.push(poses);
        }

        let mut offsets = output.iter().fold(vec![0], |mut out, vec_pose| {
            let cumulative = *out
                .last()
                .expect("init has at least one value and no values are popped");
            out.push(cumulative + vec_pose.len());
            out
        });
        let _ = offsets.pop();

        let poses = output
            .into_iter()
            .flat_map(|vec| {
                vec.into_iter().map(
                    |PoseAndInverseBindPose {
                         pose,
                         inverse_bind_pose,
                     }| pose.transform(&inverse_bind_pose),
                )
            })
            .collect();
        CalculatedAnimation { offsets, poses }
    }
}

pub(crate) fn process_vec<'a, M: AsRef<Skeleton>>(
    input: Vec<(&'a M, Pose, Option<&'a SkeletalAnimation>, f32)>,
) -> Result<Vec<(&'a M, CalculatedAnimation)>, AnimationError> {
    input
        .into_iter()
        .map(|(cubic, root, animation, time)| {
            Ok((
                cubic,
                CalculatedAnimation::builder(root, cubic.as_ref(), animation)?.build(time),
            ))
        })
        .collect()
}
