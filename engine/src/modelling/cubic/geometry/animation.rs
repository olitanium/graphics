mod bone_animation;
pub use bone_animation::BoneAnimation;

mod skeletal_animation;
pub use skeletal_animation::SkeletalAnimation;

mod calculated_animation;
pub use calculated_animation::{CalculatedAnimation, CalculatedAnimationBuilder};

mod error;
pub(crate) use calculated_animation::process_vec_noaggregate;
pub use error::Error as AnimationError;
