mod far_light;
use alloc::ffi::NulError;
use core::num::TryFromIntError;

pub use far_light::ShadowFarLight;
mod spot_light;
use graphics::shader_program::ShaderSetUniformError;
pub use spot_light::ShadowSpotLight;
mod list;
pub use list::{GenDepthError, ShadowListLights};
mod point_light;
pub use point_light::{PointLightIlluminatesGroup, ShadowPointLight};
use utils::error_boilerplate;

use crate::modelling::cubic::lighting::simple::SimpleBindError;

#[derive(Debug)]
#[non_exhaustive]
pub enum BindError {
    BindSimpleLight(SimpleBindError),
    SetUniform(ShaderSetUniformError),
    NulError(NulError),
    TooLong(<i32 as TryFrom<usize>>::Error),
}
error_boilerplate!(BindError);

impl From<ShaderSetUniformError> for BindError {
    fn from(value: ShaderSetUniformError) -> Self {
        Self::SetUniform(value)
    }
}

impl From<SimpleBindError> for BindError {
    fn from(value: SimpleBindError) -> Self {
        Self::BindSimpleLight(value)
    }
}

impl From<NulError> for BindError {
    fn from(value: NulError) -> Self {
        Self::NulError(value)
    }
}

impl From<TryFromIntError> for BindError {
    fn from(value: <i32 as TryFrom<usize>>::Error) -> Self {
        Self::TooLong(value)
    }
}
