mod far_light;
pub use far_light::ShadowFarLight;
mod spot_light;
use graphics::shader_program::SetUniformError;
pub use spot_light::ShadowSpotLight;
mod list;
pub use list::ShadowListLights;
mod point_light;
pub use point_light::ShadowPointLight;
use utils::error_boilerplate;

use crate::modelling::cubic::lighting::simple::SimpleBindError;

#[derive(Debug)]
pub enum ShadowBindError<'a> {
    BindSimpleLight(SimpleBindError<'a>),
    SetUniform(SetUniformError<'a>),
}
error_boilerplate!(<'a> ShadowBindError<'a>);

impl<'a> From<SetUniformError<'a>> for ShadowBindError<'a> {
    fn from(value: SetUniformError<'a>) -> Self {
        Self::SetUniform(value)
    }
}

impl<'a> From<SimpleBindError<'a>> for ShadowBindError<'a> {
    fn from(value: SimpleBindError<'a>) -> Self {
        Self::BindSimpleLight(value)
    }
}
