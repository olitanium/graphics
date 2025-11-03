mod animation;
mod lerp;
mod orientation;
mod pose;
mod yieldspose;

pub use animation::BoneAnimation;
pub use animation::SkeletalAnimation;
pub use lerp::Lerp;
pub use orientation::Orientation;
pub use pose::{Pose, PoseSclerp};
pub use yieldspose::{CameraPose, demo};
