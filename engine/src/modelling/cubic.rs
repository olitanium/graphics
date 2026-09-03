mod shadow;
pub use shadow::{Group as ShadowGroup, SHADOW_SHADER_MAX_LIGHTS};
mod mesh;
pub use mesh::{DrawError as MeshDrawError, Mesh, MeshBuilder};
mod import;
pub use import::{ImportError, ParseMaterialError};

mod skeleton;
pub use skeleton::{Bone, Skeleton, SkeletonBoneIndex, SkeletonBuilder};

mod camera;
pub use camera::{
    Builder as CameraBuilder,
    Camera,
    CameraPose,
    ManualPoseFixedUp,
    ManualPoseFlexible,
    Projection as CameraProjection,
};

// pub mod geometry;

pub mod lighting;

pub mod material;

mod animation;
pub use animation::*;

mod skeleton_cubic;
pub use skeleton_cubic::{
    BoneAndMesh,
    Builder as SkeletonCubicBuilder,
    DrawError as SkeletonCubicDrawError,
    Group as SkeletonCubicGroup,
    SkeletonCubic,
};

mod skinning_cubic;
pub use skinning_cubic::{
    DrawError as SkinningCubicDrawError,
    Group as SkinningCubicGroup,
    SkinningCubic,
};

mod simple_cubic;
pub use simple_cubic::{
    Builder as SimpleCubicBuilder,
    DrawError as SimpleCubicDrawError,
    Group as SimpleCubicGroup,
    SimpleCubic,
};
