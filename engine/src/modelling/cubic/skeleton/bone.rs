use crate::modelling::cubic::geometry::Pose;

mod builder;
pub use builder::Builder;

#[derive(Debug, Clone)]
pub struct Bone {
    pub(crate) parent: usize,
    inverse_bind_pose: Pose,
}

impl Bone {
    pub fn builder(parent: usize) -> Builder {
        Builder::new(parent)
    }

    pub const fn inverse_bind_pose(&self) -> Pose {
        self.inverse_bind_pose
    }
}
