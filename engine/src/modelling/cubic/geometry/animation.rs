mod bone_animation;
pub use bone_animation::BoneAnimation;

mod skeletal_animation;
pub use skeletal_animation::SkeletalAnimation;

mod calculated_animation;
pub use calculated_animation::{
    BuildFixingBoneError,
    CalculatedAnimation,
    CalculatedAnimationBuilder,
    GetAnimationError,
    PushAnimationError,
    SignaturesDoNotMatchError,
};
pub(crate) use calculated_animation::{CalcAnimUnitIndex, process_vec_noaggregate};
