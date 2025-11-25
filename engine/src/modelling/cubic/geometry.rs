mod animation;
mod lerp;
mod orientation;
mod pose;

pub(crate) use animation::process_vec;
pub use animation::{
    AnimationError,
    BoneAnimation,
    CalculatedAnimation,
    CalculatedAnimationBuilder,
    SkeletalAnimation,
};
pub use lerp::Lerp;
pub use orientation::Orientation;
pub use pose::{Pose, PoseSclerp};
