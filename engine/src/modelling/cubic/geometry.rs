mod animation;

pub use animation::{
    AggregateCalcAnim,
    AggregateCalcAnimBuilder,
    // AnimationError,
    BoneAnimation,
    CalculatedAnimation,
    SkeletalAnimation,
};
pub use animation::{
    BuildFixingBoneError,
    GetAnimationError,
    PushAnimationError,
    SignaturesDoNotMatchError,
};
pub(crate) use animation::{CalcAnimUnitIndex, process_vec_noaggregate};
