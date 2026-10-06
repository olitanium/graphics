mod far_light;
use alloc::ffi::NulError;
use core::num::TryFromIntError;

pub use far_light::FarLight;
mod list;
use graphics::shader_program::ShaderSetUniformError;
pub use list::ListLights;
mod spot_light;
pub use spot_light::SpotLight;
mod point_light;
pub use point_light::PointLight;
use utils::error_boilerplate;

#[derive(Debug)]
#[non_exhaustive]
pub enum SimpleBindError {
    SetUniform(ShaderSetUniformError),
    Nul(NulError),
    TooLong(TryFromIntError),
}
error_boilerplate!(SimpleBindError);

impl From<ShaderSetUniformError> for SimpleBindError {
    fn from(value: ShaderSetUniformError) -> Self {
        Self::SetUniform(value)
    }
}

impl From<NulError> for SimpleBindError {
    fn from(value: NulError) -> Self {
        Self::Nul(value)
    }
}

impl From<TryFromIntError> for SimpleBindError {
    fn from(value: TryFromIntError) -> Self {
        Self::TooLong(value)
    }
}
