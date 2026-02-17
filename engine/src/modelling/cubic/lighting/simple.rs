mod far_light;
pub use far_light::FarLight;
mod list;
use graphics::shader_program::SetUniformError;
pub use list::ListLights;
mod spot_light;
pub use spot_light::SpotLight;
mod point_light;
pub use point_light::PointLight;
use utils::error_boilerplate;

#[derive(Debug)]
#[non_exhaustive]
pub enum SimpleBindError<'a> {
    SetUniform(SetUniformError<'a>),
}
error_boilerplate!(<'a> SimpleBindError<'a>);

impl<'a> From<SetUniformError<'a>> for SimpleBindError<'a> {
    fn from(value: SetUniformError<'a>) -> Self {
        Self::SetUniform(value)
    }
}
