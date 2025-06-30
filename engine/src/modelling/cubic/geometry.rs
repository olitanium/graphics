mod animation;
mod lerp;
mod orientation;
mod pose;
mod yieldspose;

pub use animation::Animation;
pub use lerp::Lerp;
pub use orientation::{
    DualNumber,
    DualQuaternion,
    Orientation,
    Quaternion,
    Sclerp,
    Slerp,
    UnitDualQuaternion,
};
pub use pose::Pose;
pub use yieldspose::{demo, YieldsPose};
