mod animation;
mod lerp;
mod orientation;
mod pose;

pub use animation::{
    // AnimationError,
    BoneAnimation,
    CalculatedAnimation,
    AggregateCalcAnimBuilder,
    SkeletalAnimation,
    AggregateCalcAnim,
};
pub use animation::{
    BuildFixingBoneError,
    GetAnimationError,
    PushAnimationError,
    SignaturesDoNotMatchError,
};
pub(crate) use animation::{CalcAnimUnitIndex, process_vec_noaggregate};
pub use lerp::Lerp;
pub use orientation::Orientation;
pub use pose::{Pose, PoseSclerp};
