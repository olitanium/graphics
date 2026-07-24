mod far_light;
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
pub enum SimpleBindError<'a> {
    SetUniform(ShaderSetUniformError<'a>),
}
error_boilerplate!(<'a> SimpleBindError<'a>);

impl<'a> From<ShaderSetUniformError<'a>> for SimpleBindError<'a> {
    fn from(value: ShaderSetUniformError<'a>) -> Self {
        Self::SetUniform(value)
    }
}
