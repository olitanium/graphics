mod far_light;
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
pub enum ShadowBindError<'a> {
    BindSimpleLight(SimpleBindError<'a>),
    SetUniform(ShaderSetUniformError<'a>),
}
error_boilerplate!(<'a> ShadowBindError<'a>);

impl<'a> From<ShaderSetUniformError<'a>> for ShadowBindError<'a> {
    fn from(value: ShaderSetUniformError<'a>) -> Self {
        Self::SetUniform(value)
    }
}

impl<'a> From<SimpleBindError<'a>> for ShadowBindError<'a> {
    fn from(value: SimpleBindError<'a>) -> Self {
        Self::BindSimpleLight(value)
    }
}
