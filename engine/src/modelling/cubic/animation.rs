mod bone_animation;
pub use bone_animation::BoneAnimation;

mod skeletal_animation;
pub use skeletal_animation::SkeletalAnimation;

mod calculated_animation;
pub use calculated_animation::{
    AnimationUnit,
    CalcAnimUnitIndex,
    CalculatedAnimation,
    GetAnimationError,
    SignaturesDoNotMatchError,
    process_vec_noaggregate,
};

mod aggregate_calculated_animation;
pub use aggregate_calculated_animation::{
    AggregateCalcAnim,
    AggregateCalcAnimBuilder,
    BuildFixingBoneError,
    PushAnimationError,
};
pub use calculated_animation::{Parent, PoseAndInverseBindPose};
