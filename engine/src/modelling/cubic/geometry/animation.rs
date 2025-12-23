mod bone_animation;
pub use bone_animation::BoneAnimation;

mod skeletal_animation;
pub use skeletal_animation::SkeletalAnimation;

mod calculated_animation;
pub use calculated_animation::{
    CalculatedAnimation,
    GetAnimationError,
    SignaturesDoNotMatchError,
};
pub(crate) use calculated_animation::{CalcAnimUnitIndex, process_vec_noaggregate};
pub(crate) use calculated_animation::AnimationUnit;

mod aggregate_calculated_animation;
pub use aggregate_calculated_animation::AggregateCalcAnimBuilder;
pub use aggregate_calculated_animation::AggregateCalcAnim;
pub use aggregate_calculated_animation::BuildFixingBoneError;
pub use aggregate_calculated_animation::PushAnimationError;
pub(crate) use calculated_animation::Parent;
pub(crate) use calculated_animation::PoseAndInverseBindPose;