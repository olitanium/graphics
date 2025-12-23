mod bone_animation;
pub use bone_animation::BoneAnimation;

mod skeletal_animation;
pub use skeletal_animation::SkeletalAnimation;

mod calculated_animation;
pub(crate) use calculated_animation::{AnimationUnit, CalcAnimUnitIndex, process_vec_noaggregate};
pub use calculated_animation::{CalculatedAnimation, GetAnimationError, SignaturesDoNotMatchError};

mod aggregate_calculated_animation;
pub use aggregate_calculated_animation::{
    AggregateCalcAnim,
    AggregateCalcAnimBuilder,
    BuildFixingBoneError,
    PushAnimationError,
};
pub(crate) use calculated_animation::{Parent, PoseAndInverseBindPose};
