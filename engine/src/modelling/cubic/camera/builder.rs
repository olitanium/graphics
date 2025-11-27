use utils::{builder, new};

use super::{Camera, Projection};
use crate::modelling::cubic::camera::CameraPose;

#[derive(Debug, Default)]
pub struct MissingPose;

#[derive(Default, Debug)]
pub struct Builder<P> {
    pose: P,
    radius: f32,
    projection: Option<Projection>,
}

impl Builder<MissingPose> {
    new!();
}

impl<P> Builder<P> {
    builder!(radius: f32);

    pub fn pose<PN: CameraPose>(self, pose: PN) -> Builder<PN> {
        Builder { pose, ..self }
    }

    pub const fn perspective(mut self, fov: f32, aspect: f32, near: f32, far: f32) -> Self {
        self.projection = Some(Projection::Perspective {
            fov,
            aspect,
            near,
            far,
        });
        self
    }

    pub const fn orthographic(mut self, width: f32, height: f32, near: f32, far: f32) -> Self {
        self.projection = Some(Projection::Orthographic {
            width,
            height,
            near,
            far,
        });
        self
    }
}

impl<P: CameraPose> Builder<P> {
    pub fn build(self) -> Camera<P> {
        Camera {
            pose: self.pose,
            radius: self.radius,
            projection: self.projection.unwrap_or_default(),
        }
    }
}
