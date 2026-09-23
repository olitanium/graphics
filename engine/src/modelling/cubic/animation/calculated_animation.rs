use geometry::{Pose, Transform as _, TransformBy as _};
use utils::error_boilerplate;

use crate::entity::EntityInner;
use crate::modelling::cubic::SkeletalAnimation;
use crate::modelling::cubic::skeleton::SkeletonBoneIndex;
use crate::modelling::{CubicVertex, Skeleton};

#[derive(Debug, Clone)]
pub struct CalculatedAnimation {
    pub(crate) poses: Vec<Pose>,
    pub(crate) signature: u64,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct CalcAnimUnitIndex {
    pub(crate) index: usize,
}

impl CalcAnimUnitIndex {
    pub(crate) const ROOT_UNIT_INDEX: Self = Self { index: 0 };
}

impl CalculatedAnimation {
    pub fn new<'a>(
        root_pose: Pose,
        root_skeleton: &'a Skeleton,
        root_animation: Option<(&'a SkeletalAnimation, f32)>,
    ) -> Result<Self, SignaturesDoNotMatchError> {
        let unit = AnimationUnit::new(None, root_pose, root_skeleton, root_animation)?;
        let root = unit.local_transform;

        let (vec_pose, signature) = unit.calculate_animation(root);

        let poses = PoseAndInverseBindPose::process_vec(vec_pose);

        Ok(Self { poses, signature })
    }

    pub fn bind_pose<M: EntityInner>(model: &M, root: Pose) -> Self
    where
        M::Vertex: CubicVertex,
    {
        Self::new(root, model.skeleton(), None).expect("no animation")
    }

    pub fn poses(&self, skeleton: &Skeleton) -> Result<&[Pose], SignaturesDoNotMatchError> {
        if skeleton.signature() == self.signature {
            Ok(&self.poses)
        } else {
            Err(SignaturesDoNotMatchError)
        }
    }

    pub fn root(&self) -> Pose {
        self.poses[0]
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum GetAnimationError {
    SignaturesDoNotMatch(SignaturesDoNotMatchError),
    BoneDoesNotExist,
}

error_boilerplate!(GetAnimationError);

impl CalculatedAnimation {
    pub fn get(
        &self,
        index: &SkeletonBoneIndex,
        skeleton: &Skeleton,
    ) -> Result<Pose, GetAnimationError> {
        self.poses(skeleton)
            .map_err(GetAnimationError::SignaturesDoNotMatch)?
            .get(index.0)
            .copied()
            .ok_or(GetAnimationError::BoneDoesNotExist)
    }
}

#[derive(Debug)]
pub(crate) struct Parent {
    pub(crate) index: CalcAnimUnitIndex,
    pub(crate) bone_attachment_point: SkeletonBoneIndex,
}

/// All Animation units can only refer to units before them in the heirarchy.
/// This is enforced by the `push` method
#[derive(Debug)]
pub(crate) struct AnimationUnit<'a> {
    pub(crate) parent: Option<Parent>,
    pub(crate) local_transform: Pose,
    // invariant: Skeleton and Skeletal Animation have same signature
    pub(crate) skeleton: &'a Skeleton,
    pub(crate) animation: Option<(&'a SkeletalAnimation, f32)>,
}

#[derive(Debug)]
pub struct SignaturesDoNotMatchError;
error_boilerplate!(SignaturesDoNotMatchError);

impl<'a> AnimationUnit<'a> {
    pub(crate) const fn new(
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

    pub(crate) fn calculate_animation(self, root: Pose) -> (Vec<PoseAndInverseBindPose>, u64) {
        if let Some((animation, time)) = self.animation {
            let mut poses = vec![PoseAndInverseBindPose {
                pose: root,
                inverse_bind_pose: Pose::IDENTITY,
            }];

            for (index, bone) in self.skeleton.iter_non_root() {
                // NOTE: all bone parents are above it in the heirarchy, so we
                // can look up the total parent pose in the
                // pre-filled `poses` vec.

                let PoseAndInverseBindPose { pose: parent, .. } = *poses.get(bone.parent.0).expect(
                    "Skeleton invariant, each bone in a skeleton may only refer to bones before it",
                );
                let total = animation
                    .get_animation(index)
                    .expect("invariant, signatures are the same so should exist")
                    .get_pose(time)
                    .transform_by(parent);
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
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct PoseAndInverseBindPose {
    pub(crate) pose: Pose,
    pub(crate) inverse_bind_pose: Pose,
}

impl PoseAndInverseBindPose {
    pub(crate) fn process_vec(this: Vec<Self>) -> Vec<Pose> {
        this.into_iter()
            .map(
                |PoseAndInverseBindPose {
                     pose,
                     inverse_bind_pose,
                 }| pose.transform(inverse_bind_pose),
            )
            .collect()
    }

    pub(crate) fn process_vec_with_deviation(this: Vec<Self>, deviation: Pose) -> Vec<Pose> {
        this.into_iter()
            .map(
                |PoseAndInverseBindPose {
                     pose,
                     inverse_bind_pose,
                 }| deviation.transform(pose).transform(inverse_bind_pose),
            )
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
                CalculatedAnimation::new(root, cubic.skeleton(), animation)?,
            ))
        })
        .collect()
}

// impl From<(&CalculatedAnimation, &Skeleton)> for Pose {
// fn from(value: (&CalculatedAnimation, &Skeleton)) -> Self {
// let (anim, skeleton) = value;
// anim.get(SkeletonBoneIndex::SKELETON_ROOT_BONE, skeleton).expect("TODO: this
// is a bit yuck if skeleton isn't right") }
// }
